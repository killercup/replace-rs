//! A small crate giving you a simple container that allows easy and cheap
//! replacement of parts of its content, with the ability to prevent changing
//! the same parts multiple times.

#[deny(missing_docs)]

#[macro_use]
extern crate failure;
#[cfg(test)]
#[macro_use]
extern crate proptest;

use failure::Error;
use std::ops::Range;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Untouched,
    Touched,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Span {
    state: State,
    range: Range<usize>,
    data: Rc<[u8]>,
}

/// A container that allows easily replacing chunks of its data
#[derive(Debug, Clone, Default)]
pub struct Data {
    parts: Vec<Span>,
}

impl Data {
    /// Create a new data container from a slice of bytes
    pub fn new(data: &[u8]) -> Self {
        if data.is_empty() {
            return Data::default();
        }

        Data {
            parts: vec![Span {
                state: State::Untouched,
                range: 0..data.len(),
                data: data.into(),
            }],
        }
    }

    /// Render this data as a vector of bytes
    pub fn to_vec(&self) -> Vec<u8> {
        self.parts
            .iter()
            .map(|x| &x.data)
            .fold(Vec::new(), |mut acc, d| {
                acc.extend(d.iter());
                acc
            })
    }

    /// Replace a chunk of data with the given slice, erroring when this part
    /// was already changed previously.
    pub fn replace_range_unless_touched(
        &mut self,
        range: Range<usize>,
        data: &[u8],
    ) -> Result<(), Error> {
        self.replace_range(range, data, true)
    }

    /// Replace a chunk of data with a given slice and the option to return an
    /// error if this part of the data was already changed earlier.
    pub fn replace_range(
        &mut self,
        range: Range<usize>,
        data: &[u8],
        error_if_touched: bool,
    ) -> Result<(), Error> {
        if range.end == 0 {
            return Ok(());
        }

        let new_parts = {
            use std::cmp::min;

            let start = self.parts
                .iter()
                .position(|x| x.range.start <= range.start)
                .ok_or_else(|| format_err!("No part found that contains range {:?}", range))?;
            let end = self.parts.iter().rposition(|x| x.range.end >= range.end);

            if error_if_touched {
                let end = if let Some(end) = end {
                    end + 1
                } else {
                    self.parts.len()
                };
                let any_touched = self.parts[start..end]
                    .iter()
                    .any(|p| p.state == State::Touched);
                ensure!(
                    !any_touched,
                    "can't replace segments that were replaced previously"
                );
            }

            let mut res = Vec::with_capacity(self.parts.len());
            if start > 0 {
                res.extend(self.parts[..start.saturating_sub(1)].iter().cloned());
            }

            let start_part = &self.parts[start];

            let start_range_end = range.start.saturating_sub(start_part.range.start);

            if start_range_end > 0 {
                let data = start_part.data[..min(start_range_end, start_part.data.len())].into();
                res.push(Span {
                    state: start_part.state,
                    range: start_part.range.start..range.start,
                    data,
                });
            }

            res.push(Span {
                state: State::Touched,
                range: range.start..range.end,
                data: data.into(),
            });

            if let Some(end) = end {
                let end_part = &self.parts[end];
                if !end_part.data.is_empty() {
                    res.push(Span {
                        state: end_part.state,
                        range: range.end..end_part.range.end,
                        data: end_part.data[min(
                            range.end.saturating_sub(end_part.range.start),
                            end_part.data.len().saturating_sub(1),
                        )..]
                            .into(),
                    });

                    res.extend(self.parts[end + 1..].iter().cloned());
                }
            }

            res
        };

        self.parts = new_parts;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn str(i: &[u8]) -> &str {
        ::std::str::from_utf8(i).unwrap()
    }

    #[test]
    fn replace_some_stuff() {
        let mut d = Data::new(b"foo bar baz");

        d.replace_range(4..6, b"lol", false).unwrap();
        assert_eq!("foo lol baz", str(&d.to_vec()));

        d.replace_range(4..6, b"lol", false).unwrap();
        assert_eq!("foo lol baz", str(&d.to_vec()));

        d.replace_range(4..6, b"foobar", false).unwrap();
        assert_eq!("foo foobar baz", str(&d.to_vec()));
    }

    #[test]
    fn replace_multiple_lines() {
        let mut d = Data::new(b"lorem\nipsum\ndolor");

        d.replace_range(6..11, b"lol", false).unwrap();
        assert_eq!("lorem\nlol\ndolor", str(&d.to_vec()));

        d.replace_range(12..18, b"lol", false).unwrap();
        assert_eq!("lorem\nlol\nlol", str(&d.to_vec()));
    }

    #[test]
    fn broken_replacements() {
        let mut d = Data::new(b"foo");

        d.replace_range_unless_touched(4..7, b"lol").unwrap();
        assert_eq!("foolol", str(&d.to_vec()));
    }

    #[test]
    fn dont_replace_twice() {
        let mut d = Data::new(b"foo");

        d.replace_range_unless_touched(4..7, b"lol").unwrap();
        assert_eq!("foolol", str(&d.to_vec()));
        println!("{:?}", d);
        assert!(d.replace_range_unless_touched(4..7, b"lol").is_err());
    }

    proptest! {
        #[test]
        #[ignore]
        fn new_to_vec_roundtrip(ref s in "\\PC*") {
            assert_eq!(s.as_bytes(), Data::new(s.as_bytes()).to_vec().as_slice());
        }

        #[test]
        #[ignore]
        fn replace_random_chunks(
            ref data in "\\PC*",
            ref replacements in prop::collection::vec(
                (any::<Range<usize>>(), any::<Vec<u8>>()),
                1..1337,
            )
        ) {
            let mut d = Data::new(data.as_bytes());
            for r in replacements {
                let _ = d.replace_range(r.0.clone(), &r.1, false);
            }
        }
    }
}

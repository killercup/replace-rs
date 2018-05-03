#[macro_use]
extern crate failure;
#[cfg(test)]
#[macro_use]
extern crate proptest;

use failure::Error;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Untouched,
    Touched,
}

#[derive(Debug, Clone)]
struct Span {
    state: State,
    range: Range<usize>,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Data {
    parts: Vec<Span>,
}

impl Data {
    pub fn new(data: &[u8]) -> Self {
        if data.is_empty() {
            return Data::default();
        }

        Data {
            parts: vec![Span {
                state: State::Untouched,
                range: 0..data.len(),
                data: data.to_owned(),
            }],
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.parts
            .iter()
            .map(|x| &x.data)
            .fold(Vec::new(), |mut acc, d| {
                acc.extend(d);
                acc
            })
    }

    pub fn replace_range_unless_touched(&mut self, range: Range<usize>, data: &[u8]) -> Result<(), Error> {
        self.replace_range(range, data, true)
    }

    pub fn replace_range(&mut self, range: Range<usize>, data: &[u8], error_if_touched: bool) -> Result<(), Error> {
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
                let end = if let Some(end) = end { end + 1 } else { self.parts.len() };
                let any_touched = self.parts[start..end]
                    .iter()
                    .any(|p| p.state == State::Touched);
                ensure!(!any_touched, "can't replace segments that were replaced previously");
            }

            let mut res = Vec::with_capacity(self.parts.len());
            if start > 0 {
                res.extend(self.parts[..start.saturating_sub(1)].iter().cloned());
            }

            let start_part = &self.parts[start];

            let start_range_end = range.start.saturating_sub(start_part.range.start);

            if start_range_end > 0 {
                let data =
                    start_part.data[..min(start_range_end, start_part.data.len())].to_owned();
                res.push(Span {
                    state: start_part.state,
                    range: start_part.range.start..range.start,
                    data,
                });
            }

            res.push(Span {
                state: State::Touched,
                range: range.start..range.end,
                data: data.to_owned(),
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
                            .to_owned(),
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

    #[test]
    fn replace_some_stuff() {
        let mut d = Data::new(b"foo bar baz");

        d.replace_range(4..7, b"lol", false).unwrap();
        assert_eq!("foo lol baz".as_bytes(), d.to_vec().as_slice());

        d.replace_range(4..7, b"lol", false).unwrap();
        assert_eq!("foo lol baz".as_bytes(), d.to_vec().as_slice());

        d.replace_range(4..7, b"foobar", false).unwrap();
        assert_eq!("foo foobar baz".as_bytes(), d.to_vec().as_slice());
    }

    #[test]
    fn broken_replacements() {
        let mut d = Data::new(b"foo");

        d.replace_range_unless_touched(4..7, b"lol").unwrap();
        assert_eq!("foolol".as_bytes(), d.to_vec().as_slice());
    }

    #[test]
    fn dont_replace_twice() {
        let mut d = Data::new(b"foo");

        d.replace_range_unless_touched(4..7, b"lol").unwrap();
        assert_eq!("foolol".as_bytes(), d.to_vec().as_slice());
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

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

    pub fn replace_range(&mut self, range: Range<usize>, data: &[u8]) -> Result<(), Error> {
        let new_parts = {
            use std::cmp::min;

            let start = self.parts
                .iter()
                .position(|x| x.range.start <= range.start)
                .ok_or_else(|| format_err!("No part found that contains range {:?}", range))?;
            let end = self.parts.iter().rposition(|x| x.range.end >= range.end);

            println!("start {:?} end {:?}", start, end);

            let mut res = Vec::new();
            if start > 0 {
                res.extend(self.parts[..start - 1].iter().cloned());
            }

            let start_part = &self.parts[start];

            println!("start {:?}", start_part);
            println!("start range {:?}", ..(range.start - start_part.range.start));

            res.push(Span {
                state: start_part.state,
                range: start_part.range.start..range.start,
                data: start_part.data
                    [..min(range.start - start_part.range.start, start_part.data.len())]
                    .to_owned(),
            });

            res.push(Span {
                state: State::Touched,
                range: range.start..range.end,
                data: data.to_owned(),
            });

            if let Some(end) = end {
                let end_part = &self.parts[end];

                res.push(Span {
                    state: end_part.state,
                    range: range.end..end_part.range.end,
                    data: end_part.data[(range.end - end_part.range.start)..].to_owned(),
                });

                println!("end {:?}", end_part);
                println!("end range {:?}", (range.end - end_part.range.start)..);

                res.extend(self.parts[end + 1..].iter().cloned());
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

    proptest! {
        #[test]
        fn new_to_vec_roundtrip(ref s in "\\PC*") {
            assert_eq!(s.as_bytes(), Data::new(s.as_bytes()).to_vec().as_slice());
        }
    }

    #[test]
    fn replace_some_stuff() {
        let mut d = Data::new(b"foo bar baz");

        d.replace_range(4..7, b"lol").unwrap();
        assert_eq!("foo lol baz".as_bytes(), d.to_vec().as_slice());

        d.replace_range(4..7, b"lol").unwrap();
        assert_eq!("foo lol baz".as_bytes(), d.to_vec().as_slice());

        d.replace_range(4..7, b"foobar").unwrap();
        assert_eq!("foo foobar baz".as_bytes(), d.to_vec().as_slice());
    }

    #[test]
    fn broken_replacements() {
        let mut d = Data::new(b"foo");

        d.replace_range(4..7, b"lol").unwrap();
        assert_eq!("foolol".as_bytes(), d.to_vec().as_slice());
    }
}

#[cfg(test)] #[macro_use] extern crate proptest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum State { Untouched, Touched }

#[derive(Debug, Clone)]
struct Span {
    state: State,
    start: usize,
    end: usize,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Data {
    parts: Vec<Span>,
}

impl Data {
    pub fn new(data: &[u8]) -> Self {
        if data.is_empty() { return Data::default(); }

        Data { parts: vec![
            Span {
                state: State::Untouched,
                start: 0,
                end: data.len() - 1,
                data: data.to_owned(),
            }
        ] }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.parts.iter().map(|x| &x.data).fold(Vec::new(), |mut acc, d| { acc.extend(d); acc })
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
}

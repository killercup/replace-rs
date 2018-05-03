# replace-rs

[![Build Status](https://travis-ci.org/killercup/replace-rs.svg?branch=master)](https://travis-ci.org/killercup/replace-rs)

A small crate giving you a simple container
that allows easy and cheap replacement of parts of its content,
with the ability to prevent changing the same parts multiple times.


## Examples

```rust
extern crate replace;

fn main() {
    let mut d = Data::new(b"foo bar baz");

    d.replace_range(4..7, b"lol", false).unwrap();
    assert_eq!("foo lol baz".as_bytes(), d.to_vec().as_slice());
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

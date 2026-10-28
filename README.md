# ns_keyed_archive

A library to encode/decode NSKeyedArchive-formatted binary plists
Based off the work from [NSKeyedArchiver Converter](https://github.com/michaelwright235/nskeyedarchiver_converter)

## Usage

Add this library and ``plist`` to your ``Cargo.toml``

```toml
[dependencies]
plist = "1.7"
ns_keyed_archive = "*"
```

```rust
use plist;

// Create a new plist and insert values
let mut d = plist::Dictionary::new();
let a = plist::Value::Array(vec!["a".into(), 1.into()]);
d.insert("reeeeee".to_string(), a);
d.insert("asdf".into(), "qwer".into());

let mut d_d = plist::Dictionary::new();
d_d.insert("fortnite".into(), plist::Value::Boolean(false));
d.insert("good games".into(), d_d.into());

// Encode the binary in an archived binary plist
let b = encode::encode_to_bytes(d.clone().into()).unwrap();

// Decode it again
let d1 = decode::from_bytes(&b).unwrap();
assert_eq!(plist::Value::Dictionary(d), d1);

// Another example
let s = plist::Value::String("asdf".into());
let b = encode::encode_to_bytes(s.clone()).unwrap();
let s1 = decode::from_bytes(&b).unwrap();
assert_eq!(s, s1);
```

## License

Copyright (c) 2025 Jackson Coxson

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

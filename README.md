# P.O.T.
## Parser of Toml powerd by  [Muncher](https://github.com/DevinR528/muncher)

[![Build Status](https://travis-ci.com/DevinR528/toml-parse.svg?branch=master)](https://travis-ci.com/DevinR528/toml-parse)
<!-- [![Latest Version](https://img.shields.io/crates/v/par-trie.svg)](https://crates.io/crates/toml) -->

## About
The most important thing about this toml parser is that it maintains the structure of the original parsed file. For toml formatting tools like [cargo sort check](https://github.com/DevinR528/cargo-sort-ck) this feature is essential, this is the only reason something like [toml-rs](https://github.com/alexcrichton/toml-rs/tree/0.4.6) cannot be used unfortunatly :(. 

## Use
```toml
[dependencies]
par-trie = "0.1"
```

## Examples
```rust
use pot::{Toml, Value};

let file = 
r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;

let parsed = Toml::parse(file).unwrap();
parsed.get_table("deps").add("new-key", "hello");
assert!(parsed.get_table("deps").contains_key("new-key"));
```


#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>

# P.O.T.
## Parser of Toml powerd by  [Muncher](https://github.com/DevinR528/muncher)

[![Build Status](https://travis-ci.com/DevinR528/toml-parse.svg?branch=master)](https://travis-ci.com/DevinR528/toml-parse)
[![Latest Version](https://img.shields.io/crates/v/toml-parse.svg)](https://crates.io/crates/toml-parse)

[Documentation](https://docs.rs/toml-parse)

## About
The most important thing about this toml parser is that it maintains the structure of the original parsed file (whitespace, comments, ect.). For toml formatting tools like [cargo sort check](https://github.com/DevinR528/cargo-sort-ck) this feature is essential, this is the only reason something like [toml-rs](https://github.com/alexcrichton/toml-rs/tree/0.4.6) cannot be used unfortunatly :(. 

## Use
```toml
[dependencies]
toml-parse = "0.1"
```

## Examples

### Parsing
```rust
use toml_parse::{parse_it, SyntaxNodeExtTrait};

let file = 
r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;

let parsed = parse_it(file).expect("parse failed");
let root = parsed.syntax();
assert_eq!(root.token_text(), file)
```
The parse tree is a [`rowan`](https://docs.rs/rowan/0.9.1/rowan/) `SyntaxNode` that can be manipulated and traversed freely.
The `SyntaxNodeExtTrait` allows easy to string representation of the tokens (the source file text).
### Sorting
```rust
use toml_parse::{parse_it, SyntaxNodeExtTrait};

let file = r#"# comment
[dependencies]
number = 1234
# comment
alpha = "beta"
"#;
let parsed = parse_it(file).expect("parse failed").syntax();
let parsed2 = parse_it(file).expect("parse failed").syntax();

assert!(parsed.deep_eq(&parsed2));

let sorted = sort_toml_items(&parsed, &HEADER);

assert!(!parsed.deep_eq(&sorted));
```
The sorted tree is a [`rowan`](https://docs.rs/rowan/0.9.1/rowan/) `SyntaxNode` that can be manipulated and traversed freely.


### Formatting
```rust
use toml_parse::{parse_it, Formatter};

let parsed = parse_it(&input).expect("").syntax();
let fmted = Formatter::new(&parsed).format();
assert_ne!(fmted.to_string(), input);
```

### Structured (Coming soonish)

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

use std::str::FromStr;

use chrono::{Date, DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime};

mod date;
mod err;
mod parse;
mod token;
mod value;
mod table;

use err::{ParseTomlError, TomlErrorKind, TomlResult};
use parse::Parse;
use token::{cmp_tokens, Muncher};
use value::Value;

pub(crate) const EOL: &[char] = &['\n', '\r'];
pub(crate) const DATE_END: &[char] = &['\n', '\r', ',', ']' ];
pub(crate) const NUM_END: &[char] = &['\n', '\r', ',', ']', ' ' ];
pub(crate) const ARRAY_ITEMS: &[char] = &[ ',', ']' ];
pub(crate) const OBJ_ITEMS: &[char] = &[ ',', ' ', '}' ];
pub(crate) const BOOL: &[char] = &['t', 'f'];
pub(crate) const KEY_END: &[char] = &['\n', '\r', ' ', ',', '='];
pub(crate) const DATE_LIKE: &[char] = &['-', '/', ':', 'T'];
pub(crate) const SPACE_EQ: &[char] = &[' ', '='];


#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use super::*;
    
    #[test]
    fn kv_table() {
        let input = r#"[hello]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");

        if let Value::Table(table) = value {
            assert_eq!(table.header(), "hello");
            assert_eq!(table.item_len(), 2);
        } else {
            panic!("no table parsed")
        }
    }

    #[test]
    fn seg_header() {
        let input = r#"[hello.world]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");
        println!("{:#?}", value);
        if let Value::Table(table) = value {
            assert_eq!(table.header(), "hello.world");
            assert_eq!(table.seg_len(), 2);
            assert_eq!(table.item_len(), 2);
        } else {
            panic!("no table parsed")
        }
    }

    #[test]
    fn file_parser() {
        let input = read_to_string("examp/ftop.toml").expect("file read failed");

        let mut parsed = Vec::default();
        let mut muncher = Muncher::new(&input);
        while let Ok(value) = Value::parse(&mut muncher) {
            println!("{:#?}", value);
            parsed.push(value);
        }
    }
}

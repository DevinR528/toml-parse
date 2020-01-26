use std::str::FromStr;

use chrono::{Date, DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime};

mod date;
mod err;
mod parse;
mod token;
mod value;
mod table;

pub use err::{ParseTomlError, TomlErrorKind, TomlResult};
use parse::Parse;
use token::{cmp_tokens, Muncher};
pub use value::Value;
pub use table::{Heading, KvPairs, InTable, Table};

pub(crate) const EOL: &[char] = &['\n', '\r'];
pub(crate) const DATE_END: &[char] = &['\n', '\r', ',', ']' ];
pub(crate) const NUM_END: &[char] = &['\n', '\r', ',', ']', ' ' ];
pub(crate) const ARRAY_ITEMS: &[char] = &[ ',', ']' ];
pub(crate) const KEY_END: &[char] = &[ ' ', ',', '='];
pub(crate) const DATE_LIKE: &[char] = &['-', '/', ':', 'T'];

pub struct TomlFile {
    items: Vec<Value>,
}

impl Parse for TomlFile {
    type Item = TomlFile;
    fn parse(muncher: &mut Muncher) -> TomlResult<Self> {
        let mut parsed = Vec::default();
        while let Ok(value) = Value::parse(muncher) {
            if value == Value::Eof { break };
            parsed.push(value);
        }
        Ok(TomlFile { items: parsed, }) 
    }
}

pub struct Toml(TomlFile);

impl Toml {
    pub fn parse(input: &str) -> TomlResult<Toml> {
        let mut muncher = Muncher::new(input);
        <Toml as Parse>::parse(&mut muncher)
    }

    /// The number of items found in a parsed toml file.
    pub fn len(&self) -> usize {
        self.0.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_table(&self, heading: &str) -> Option<table::Table> {
        None
    }

    pub fn get_value(&self, key: &str) -> Option<Value> {
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.0.items.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.0.items.iter_mut()
    }
}

impl Parse for Toml {
    type Item = Toml;
    fn parse(muncher: &mut Muncher) -> TomlResult<Self> {
        Ok(Toml(TomlFile::parse(muncher)?))
    }
}

impl IntoIterator for Toml {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use super::*;
    #[test]
    fn toml_parser() {
        // ftop.toml is 7 items long
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let parsed = Toml::parse(&input).expect("parse failed");
        assert_eq!(parsed.len(), 7);
        for item in parsed {
            println!("{:#?}", item);
        }
    }
}

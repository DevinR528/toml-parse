mod date;
mod parse;
mod table;
mod value;

pub(self) use super::common::{
    err::{self, ParseTomlError, TomlErrorKind, TomlResult},
    munch::{
        self, Muncher, ARRAY_ITEMS, BOOL_END, DATE_CHAR, DATE_LIKE, EOL, KEY_END, NUM_END,
        TIME_CHAR,
    },
};

use parse::Parse;
pub use table::{Heading, InTable, KvPairs, Table};
pub use value::Value;

// pub(crate) const EOL: &[char] = &['\n', '\r'];
// pub(crate) const NUM_END: &[char] = &['\n', '\r', ',', ']', ' ', '}'];
// pub(crate) const BOOL_END: &[char] = &['\n', '\r', ',', ']', ' ', '}'];
// pub(crate) const ARRAY_ITEMS: &[char] = &[',', ']'];
// pub(crate) const KEY_END: &[char] = &[' ', ',', '='];
// pub(crate) const DATE_LIKE: &[char] = &['-', '/', ':', 'T'];

#[derive(Debug)]
pub struct Toml {
    items: Vec<Value>,
}

impl Toml {
    pub fn parse(input: &str) -> TomlResult<Toml> {
        let mut muncher = Muncher::new(input);
        <Toml as Parse>::parse(&mut muncher)
    }

    /// The number of items found in a parsed toml file.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a whole table that matches `heading`.
    pub fn get_table(&self, heading: &str) -> Option<&table::Table> {
        self.iter()
            .find(|val| match val {
                Value::Table(tab) => tab.header() == heading,
                _ => false,
            })
            .map(|table| {
                if let Value::Table(tab) = table {
                    Some(tab)
                } else {
                    None
                }
            })
            .flatten()
    }

    /// Returns any bare value with `key` as its key.
    ///
    /// ```ignore
    /// // no header these key value pairs are not part of a table.
    /// key = value
    /// ```
    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.iter().find(|val| match val {
            Value::KeyValue(kv) => kv.key() == Some(key),
            _ => false,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.items.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.items.iter_mut()
    }
}

impl Parse for Toml {
    type Item = Toml;
    fn parse(muncher: &mut Muncher) -> TomlResult<Self> {
        let mut parsed = Vec::default();
        loop {
            let value = Value::parse(muncher)?;
            if value == Value::Eof {
                break;
            };
            parsed.push(value);
        }
        Ok(Toml { items: parsed })
    }
}

impl IntoIterator for Toml {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
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

    #[test]
    fn all_value_types() {
        let file = r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;
        let parsed = Toml::parse(file).expect("parse failed");
        assert_eq!(parsed.len(), 1);
        let tab = parsed.get_table("deps").unwrap();
        assert_eq!(tab.header(), "deps");
        assert_eq!(tab.get("number").unwrap().value(), &Value::Int(1234));
    }
}

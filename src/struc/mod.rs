pub(self) use super::common::{err, munch};
pub(self) use super::tkn_tree;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tkn_tree::{
    parse_it,
    walk::{walk_nodes, walk_non_whitespace, walk_tokens},
    SyntaxElement, SyntaxNode, SyntaxToken,
    TomlKind::*,
};

#[derive(Debug)]
pub struct Toml {
    items: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    header: Heading,
    pairs: Vec<KvPairs>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    header: String,
    seg: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InTable {
    pairs: Vec<KvPairs>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KvPairs {
    key: Option<String>,
    val: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    StrLit(String),
    Date(TomlDate),
    Array(Vec<Value>),
    InlineTable(InTable),
    Table(Table),
    Comment(String),
    KeyValue(Box<KvPairs>),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TomlDate {
    DateTime(NaiveDateTime),
    Date(NaiveDate),
    Time(NaiveTime),
}

impl Toml {
    pub fn new(input: &str) -> Toml {
        let root = parse_it(input).expect("parse failed").syntax();
        dbg!(root);
        Self {
            items: Vec::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_small() {
        let file = "[table]\nkey = \"value\"";

        let toml = Toml::new(file);
    }

    #[test]
    fn into_structured() {
        let file = r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;

        let toml = Toml::new(file);
    }
}

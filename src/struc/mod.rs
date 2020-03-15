use std::cmp::Ordering;

pub(self) use super::common::{self, err};
pub(self) use super::tkn_tree;

use tkn_tree::{parse_it, SyntaxElement, SyntaxNode, SyntaxNodeExtTrait, SyntaxToken, TomlKind};

mod date;
use date::TomlDate;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toml {
    items: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Table {
    comment: Option<String>,
    header: Heading,
    pairs: Vec<KvPair>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Heading {
    header: String,
    seg: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct InTable {
    pairs: Vec<KvPair>,
    trailing_comma: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct KvPair {
    comment: Option<String>,
    key: Option<String>,
    val: Value,
}

impl Ord for KvPair {
    fn cmp(&self, other: &KvPair) -> Ordering {
        match self.key() {
            Some(key) => match other.key() {
                Some(k) => key.cmp(k),
                None => Ordering::Equal,
            },
            None => Ordering::Equal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
    KeyValue(Box<KvPair>),
    Root,
    Eof,
    None,
}

impl Eq for Value {}

fn strip_start_end(mut input: String, count: usize) -> String {
    for _ in 0..count {
        input.remove(0);
        input.pop();
    }
    input
}

fn integer(s: &str) -> err::TomlResult<Value> {
    let s = s.replace('_', "");
    let cleaned = s.trim_start_matches('+');
    if s.starts_with("0x") {
        let without_prefix = cleaned.chars().skip(2).collect::<String>();
        let z = i64::from_str_radix(&without_prefix, 16);
        Ok(Value::Int(z?))
    } else if s.starts_with("0o") {
        let without_prefix = cleaned.chars().skip(2).collect::<String>();
        let z = i64::from_str_radix(&without_prefix, 8);
        Ok(Value::Int(z?))
    } else if s.starts_with("0b") {
        let without_prefix = cleaned.chars().skip(2).collect::<String>();
        let z = i64::from_str_radix(&without_prefix, 2);
        Ok(Value::Int(z?))
    } else {
        Ok(Value::Int(cleaned.parse()?))
    }
}

fn ws(node: &SyntaxNode) -> bool {
    node.kind() != TomlKind::KeyValue
}

fn ws_ele(node: &SyntaxElement) -> bool {
    node.kind() != TomlKind::KeyValue
}

fn first_child_not_ws(node: SyntaxNode) -> Option<SyntaxElement> {
    node.children_with_tokens().find(ws_ele)
}

impl Value {
    // NODES
    fn node_to_value(node: SyntaxNode) -> Value {
        if let Some(val) = node.first_child().map(|n| n.into()) {
            return val;
        }
        // else child is token such as ident, integer, date, ect.
        first_child_not_ws(node)
            .map(|n| n.as_token().map(|t| t.clone().into()))
            .flatten()
            .unwrap()
    }
    fn node_to_array(node: SyntaxNode) -> Value {
        let array = node.children().filter(ws).map(|n| n.into()).collect();
        Value::Array(array)
    }
    fn node_to_array_item(node: SyntaxNode) -> Value {
        node.first_child().map(|n| n.into()).unwrap()
    }
    fn node_to_comment(node: SyntaxNode) -> Value {
        Value::Comment(node.token_text())
    }
    fn node_to_string(node: SyntaxNode) -> Value {
        let mut string = node.token_text();
        if string.starts_with("\"\"\"") {
            string = strip_start_end(string, 3);
        } else if string.starts_with('\"') || string.starts_with('\'') {
            string = strip_start_end(string, 1);
        }
        Value::StrLit(string)
    }
    fn node_to_float(node: SyntaxNode) -> Value {
        let float = node.token_text();
        let cleaned = float.replace('_', "");
        Value::Float(cleaned.parse().unwrap())
    }
    fn node_to_date(tkn: SyntaxNode) -> Value {
        let raw_date = tkn.token_text();
        let date = TomlDate::from_str(&raw_date);
        Value::Date(date.unwrap())
    }

    // TOKENS
    fn token_to_int(tkn: SyntaxToken) -> Value {
        let int = tkn.text();
        integer(&int).unwrap()
    }
    fn token_to_bool(tkn: SyntaxToken) -> Value {
        let raw_bool = tkn.text();
        if raw_bool == "true" {
            Value::Bool(true)
        } else {
            Value::Bool(false)
        }
    }
}

impl Into<Table> for SyntaxNode {
    fn into(self) -> Table {
        let header = self.first_child().map(|n| n.into()).unwrap();
        let pairs = self.children().skip(1).map(|n| n.into()).collect();
        Table {
            header,
            pairs,
            comment: None,
        }
    }
}

impl Into<Heading> for SyntaxNode {
    fn into(self) -> Heading {
        let mut header = self.token_text();
        if header.contains('[') {
            header = header.split('[').collect::<Vec<_>>()[1].to_string();
        }
        if header.contains(']') {
            header = header.split(']').collect::<Vec<_>>()[0].to_string();
        }
        let seg = header.split('.').map(|s| s.into()).collect::<Vec<_>>();

        Heading { header, seg }
    }
}

impl Into<InTable> for SyntaxNode {
    fn into(self) -> InTable {
        let pairs = self.children().map(|n| n.into()).collect();
        let trailing_comma = false;
        InTable {
            pairs,
            trailing_comma,
        }
    }
}

impl Into<KvPair> for SyntaxNode {
    fn into(self) -> KvPair {
        if self.kind() == TomlKind::Comment {
            return KvPair {
                key: None,
                val: Value::None,
                comment: Some(self.token_text()),
            };
        }

        let key = self.first_child().map(|n| n.token_text());

        let val = self
            .children()
            .find(|n| n.kind() == TomlKind::Value || n.kind() == TomlKind::Comment)
            .filter(ws)
            .map(|n| n.into())
            .unwrap_or(Value::Eof);

        KvPair {
            key,
            val,
            comment: None,
        }
    }
}

impl Into<Value> for SyntaxNode {
    fn into(self) -> Value {
        // println!("INTO {:#?}", self);
        match self.kind() {
            TomlKind::Root => Value::Root,
            TomlKind::Table => Value::Table(self.into()),
            TomlKind::KeyValue => Value::KeyValue(Box::new(self.into())),
            TomlKind::InlineTable => Value::InlineTable(self.into()),
            TomlKind::Array => Value::node_to_array(self),
            TomlKind::ArrayItem => Value::node_to_array_item(self),
            TomlKind::Value => Value::node_to_value(self),
            TomlKind::Date => Value::node_to_date(self),
            TomlKind::Comment => Value::node_to_comment(self),
            TomlKind::Str => Value::node_to_string(self),
            TomlKind::Float => Value::node_to_float(self),
            _ => unreachable!("may need to add nodes"),
        }
    }
}

impl Into<Value> for SyntaxToken {
    fn into(self) -> Value {
        match self.kind() {
            TomlKind::Integer => Value::token_to_int(self),
            TomlKind::Bool => Value::token_to_bool(self),
            _ => unreachable!("may need to add nodes"),
        }
    }
}

impl Toml {
    /// Create structured toml objects from valid toml `&str`
    pub fn new(input: &str) -> Toml {
        let root = parse_it(input).expect("parse failed").syntax();
        Self {
            items: root.children().map(|node| node.into()).collect(),
        }
    }
}

impl Value {
    /// Returns a reference to `InTable` if `self` is an
    /// inline table, otherwise None.
    pub fn as_inline_table(&self) -> Option<&InTable> {
        match self {
            Value::InlineTable(table) => Some(table),
            _ => None,
        }
    }
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }
    pub fn as_key_value(&self) -> Option<&KvPair> {
        match self {
            Value::KeyValue(kv) => Some(kv),
            _ => None,
        }
    }
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn sort_string_array(&mut self) {
        if let Value::Array(array) = self {
            let all_str = array.iter().all(|item| match item {
                Value::StrLit(_) => true,
                _ => false,
            });

            if !all_str {
                return;
            }

            array.sort_by(|item, other| match item {
                Value::StrLit(s) => match other {
                    Value::StrLit(o) => s.cmp(o),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            })
        }
    }
}

impl KvPair {
    fn key_match(&self, key: &str) -> bool {
        self.key.as_ref().map(|k| k == key) == Some(true)
    }

    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }
    pub fn value(&self) -> &Value {
        &self.val
    }
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.val
    }
}

impl Table {
    /// The heading of the given `Table`.
    pub fn header(&self) -> &str {
        &self.header.header
    }
    /// The segments of the heading of a given `Table`.
    pub fn segments(&self) -> &[String] {
        &self.header.seg
    }
    /// The number of items in this `Table`.
    pub fn item_len(&self) -> usize {
        self.pairs.len()
    }
    /// Number of segments the header is broken into.
    ///
    /// ```ignore
    /// [this.is.segmented]
    /// key = "value"
    /// ```
    pub fn seg_len(&self) -> usize {
        self.header.seg.len()
    }
    /// The `KvPair` this table holds as a slice.
    pub fn items(&self) -> &[KvPair] {
        &self.pairs
    }

    /// Returns `KvPair` that matches given key.
    pub fn get_key_value(&self, key: &str) -> Option<&KvPair> {
        self.pairs.iter().find(|pair| pair.key_match(key))
    }
    /// Returns `Value` that matches given key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.pairs
            .iter()
            .find(|pair| pair.key_match(key))
            .map(|pair| pair.value())
    }

    /// Returns `Value` that matches given key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.pairs
            .iter_mut()
            .find(|pair| pair.key_match(key))
            .map(|pair| pair.value_mut())
    }

    pub fn sort(&mut self) {
        self.pairs.sort()
    }

    /// Merges `Value::Comment` with `Value` below it.
    #[allow(clippy::while_let_loop)]
    pub fn combine_comments(&mut self) {
        {
            let cmt_clone = self.clone();
            let mut zipped = cmt_clone.iter().zip(self.iter_mut().skip(1)).peekable();

            loop {
                if let Some((left, right)) = zipped.next() {
                    if let Some(comment) = &left.comment {
                        right.comment = Some(comment.into());
                    }
                } else {
                    break;
                }
            }
        }
        self.pairs
            .retain(|kv| !(kv.key().is_none() && kv.value() == &Value::None))
    }

    pub fn iter(&self) -> impl Iterator<Item = &KvPair> {
        self.pairs.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut KvPair> {
        self.pairs.iter_mut()
    }
}

impl InTable {
    /// Number of `KvPair's in given inline table.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }
    /// Returns true if `InTable` has no `KvPair`s.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the `Value` that matches given key.
    ///
    /// # Example
    /// ```
    /// use toml_parse::{Toml, Value};
    ///
    /// let input = "examp = { first = 1, second = 2 }";
    /// let toml = Toml::new(input);
    ///
    /// if let Some(Value::InlineTable(inline)) = toml.get_bare_value("examp") {
    ///     assert_eq!(inline.get("second"), Some(&Value::Int(2)));
    ///     assert_eq!(inline.get("first"), Some(&Value::Int(1)));
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.pairs
            .iter()
            .find(|pair| pair.key_match(key))
            .map(|pair| pair.value())
    }
}

impl Toml {
    /// The number of items found in a parsed toml file.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the `Table` that matches `heading`.
    pub fn get_table(&self, heading: &str) -> Option<&Table> {
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

    /// Returns a mutable `Table` that matches `heading`.
    pub fn get_table_mut(&mut self, heading: &str) -> Option<&mut Table> {
        self.iter_mut()
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

    /// Returns a mutable `Table` that contains `heading` or heading fragment.
    pub fn get_contains_mut(&mut self, heading: &str) -> Vec<&mut Table> {
        self.iter_mut()
            .filter(|val| match val {
                Value::Table(tab) => tab.header().contains(heading),
                _ => false,
            })
            .flat_map(|table| {
                if let Value::Table(tab) = table {
                    Some(tab)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns any bare value with `key` as its key.
    ///
    /// ```ignore
    /// // no header these key value pairs are not part of a table.
    /// key = value
    /// ```
    pub fn get_bare_value(&self, key: &str) -> Option<&Value> {
        self.iter().find(|val| match val {
            Value::KeyValue(kv) => kv.key() == Some(key),
            _ => false,
        })
    }

    pub fn sort_matching(&mut self, heading: &str) {
        self.items.sort_by(|tab, other| match tab {
            Value::Table(tab) => {
                if tab.header().contains(heading) {
                    match other {
                        Value::Table(other) => {
                            if other.header().contains(heading) {
                                tab.segments().last().cmp(&other.segments().last())
                            } else {
                                Ordering::Equal
                            }
                        }
                        _ => Ordering::Equal,
                    }
                } else {
                    Ordering::Equal
                }
            }
            _ => Ordering::Equal,
        })
    }

    /// Merges `Value::Comment` with `Value` below it.
    #[allow(clippy::while_let_loop)]
    pub fn combine_comments(&mut self) {
        {
            let cmt_clone = self.clone();
            let mut zipped = cmt_clone.iter().zip(self.iter_mut().skip(1)).peekable();

            loop {
                if let Some((left, right)) = zipped.next() {
                    if let Value::Comment(comment) = left {
                        match right {
                            Value::Table(t) => {
                                t.combine_comments();
                                t.comment = Some(comment.into())
                            }
                            Value::KeyValue(kv) => kv.comment = Some(comment.into()),
                            Value::Comment(cmt) => cmt.push_str(&format!("{}\n", comment)),
                            _ => unreachable!("only kv, comments and tables"),
                        }
                    }
                } else {
                    break;
                }
            }
        }
        self.items.retain(|val| {
            if let Value::Comment(_) = val {
                false
            } else {
                true
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.items.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.items.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn comment() {
        let file = r#"# comment
[deps]
number = 1234
# comment
alpha = "beta"
"#;
        let mut toml = Toml::new(file);
        toml.combine_comments();
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
        assert!(toml.get_table("deps").is_some());
    }

    #[test]
    fn ftop_file_struc() {
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let parsed = Toml::new(&input);

        assert_eq!(parsed.len(), 5);
    }
    #[test]
    fn fend_file_struc() {
        let input = read_to_string("examp/fend.toml").expect("file read failed");
        let parsed = Toml::new(&input);

        assert_eq!(parsed.len(), 6);
        // println!("{:#?}", parsed.len());
    }
    #[test]
    fn seg_file_struc() {
        let input = read_to_string("examp/seg.toml").expect("file read failed");
        let parsed = Toml::new(&input);

        assert_eq!(parsed.len(), 2);
        // println!("{:#?}", parsed.len());
    }
    #[test]
    fn work_file_struc() {
        let input = read_to_string("examp/work.toml").expect("file read failed");
        let parsed = Toml::new(&input);
        let members = parsed
            .get_table("workspace")
            .unwrap()
            .get("members")
            .unwrap();

        assert_eq!(members.as_array().unwrap().len(), 4);
    }

    #[test]
    fn all_value_types() {
        let file = r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;
        let parsed = Toml::new(file);
        assert_eq!(parsed.len(), 1);
        let tab = parsed.get_table("deps").unwrap();
        assert_eq!(tab.header(), "deps");
        assert_eq!(tab.get("number").unwrap(), &Value::Int(1234));
    }

    #[test]
    fn docs() {
        let input = "examp = { first = 1, second = 2 }";
        let toml = Toml::new(input);
        // println!("{:#?}", toml);
        if let Some(Value::KeyValue(kv)) = toml.get_bare_value("examp") {
            let inline = kv.value().as_inline_table();
            assert_eq!(inline.unwrap().get("second"), Some(&Value::Int(2)));
            assert_eq!(inline.unwrap().get("first"), Some(&Value::Int(1)));
        } else {
            panic!("bare key value not found")
        }
    }

    #[test]
    fn merge_comments_ftop() {
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let mut parsed = Toml::new(&input);
        parsed.combine_comments();
        let parse_cmp = parsed.clone();
        assert_eq!(parsed, parse_cmp);
        {
            let deps = parsed.get_table_mut("dependencies").unwrap();
            deps.sort();
        }
        parsed.sort_matching("dependencies.");
        // println!("{:#?}", parsed);
        assert_ne!(parsed, parse_cmp);
    }

    #[test]
    fn sort_ftop() {
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let mut parsed = Toml::new(&input);
        let parse_cmp = parsed.clone();
        assert_eq!(parsed, parse_cmp);
        {
            let deps = parsed.get_table_mut("dependencies").unwrap();
            deps.sort();
        }
        parsed.sort_matching("dependencies.");
        // println!("{:#?}", parsed);
        assert_ne!(parsed, parse_cmp);
    }
    #[test]
    fn sort_fend() {
        let input = read_to_string("examp/fend.toml").expect("file read failed");
        let mut parsed = Toml::new(&input);

        let parse_cmp = parsed.clone();
        assert_eq!(parsed, parse_cmp);
        {
            let deps = parsed.get_table_mut("dependencies").unwrap();
            deps.sort();
        }
        parsed.sort_matching("dependencies.");
        // println!("{:#?}", parsed);
        assert_ne!(parsed, parse_cmp);
    }

    #[test]
    fn sort_win() {
        let input = read_to_string("examp/win.toml").expect("file read failed");
        let mut parsed = Toml::new(&input);
        let parse_cmp = parsed.clone();
        assert_eq!(parsed, parse_cmp);
        {
            // sorts items of a table
            let deps = parsed.get_table_mut("dependencies").unwrap();
            deps.sort();
        }
        // sorts tables by last segment
        parsed.sort_matching("dependencies.");
        assert_ne!(parsed, parse_cmp);
    }

    #[test]
    fn sort_work() {
        let input = read_to_string("examp/work.toml").expect("file read failed");
        let mut parsed = Toml::new(&input);
        let members = parsed
            .get_table_mut("workspace")
            .unwrap()
            .get_mut("members")
            .unwrap();
        let mut mem_cmp = members.clone();
        assert_eq!(*members, mem_cmp);
        members.sort_string_array();
        assert_ne!(*members, mem_cmp);
        mem_cmp.sort_string_array();
        assert_eq!(*members, mem_cmp);
    }
}

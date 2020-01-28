use smol_str::{SmolStr};
use std::ops::Range;

type TextRange = Range<usize>;

#[derive(Clone, Debug)]
pub struct TomlNode {
    kind: TomlKind,
    text: SmolStr,
    children: Vec<Element>,
    range: TextRange,
}

impl TomlNode {
    // pub fn new(text: &str, range: TextRange) -> Self {
        
    // }

    pub fn kind(&self) -> TomlKind {
        self.kind
    }
}

#[derive(Clone, Debug)]
pub struct TomlToken {
    kind: TomlKind,
    text: SmolStr,
    range: TextRange,
}

impl TomlToken {
    // pub fn new(text: &str, range: TextRange) -> Self {
        
    // }

    pub fn kind(&self) -> TomlKind {
        self.kind
    }
}

/// An enum representing either a `Node` or a `Token`.
#[derive(Clone, Debug)]
pub enum Element {
    Node(TomlNode),
    Token(TomlToken),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomlKind {
    // these are nodes
    /// the "empty" root node representing a whole file.
    Root,
    /// A toml array.
    Array,
    /// A toml table consisting of a heading and key
    /// value pairs.
    Table,
    /// An inline table where the key is the "heading" and
    /// key value pairs inside of curly braces.
    InlineTable,
    /// A key and a value, any other valid toml type.
    KeyValue,
    /// Any valid toml type after a key.
    Value,
    /// A table heading surounded by brackets.
    Heading,
    /// One of three string types, literal single quote,
    /// normal double quote and literal triple double quote.
    /// (like python doc comments)
    Str,
    /// A key either `Ident` or double quoted.
    Key,
    /// A comment in the toml file, a `Hash` token followed by `CommentText`.
    Comment,

    // these are considered tokens from
    // here down
    /// the text of a comment.
    CommentText,
    /// Toml date
    /// TODO this is one of with offset, without, local,
    /// time, date and datetime.
    Date,
    /// A signed 64 bit number.
    Integer,
    /// An signed 64 bit EEE 754-2008 "binary64" number.
    Float,
    /// True or false.
    Bool,
    /// The token when a key is not surrounded by quotes.
    Ident,

    /// Single quote.
    Quote,
    /// Double quote, used for keys and strings.
    DoubleQuote,

    /// 
    Plus,
    ///
    Minus,
    ///
    Equal,
    ///
    Hash,

    ///
    Dot,
    ///
    Comma,
    ///
    Colon,

    ///
    OpenCurly,
    ///
    CloseCurly,
    ///
    OpenBrace,
    ///
    CloseBrace,

    /// All whitespace tokens, newline, indent,
    /// space and tab are all represented by this token.
    Whitespace,
}

pub enum Token {
    
}

impl Element {
    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::Node(_) => false,
            Self::Token(tkn) => tkn.kind() == TomlKind::Whitespace,
        }
    }
}

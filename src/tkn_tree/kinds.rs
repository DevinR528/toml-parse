use rowan::SmolStr;
use std::ops::Range;

type TextRange = Range<usize>;

/// An enum representing either a `Node` or a `Token`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Element {
    Node(TomlNode),
    Token(TomlToken),
}

impl PartialEq<TomlNode> for Element {
    fn eq(&self, other: &TomlNode) -> bool {
        match self {
            Element::Node(n) => n == other,
            Element::Token(_) => false,
        }
    }
}

impl PartialEq<TomlToken> for Element {
    fn eq(&self, other: &TomlToken) -> bool {
        match self {
            Element::Token(tkn) => tkn == other,
            Element::Node(_) => false,
        }
    }
}

impl Element {
    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::Node(_) => false,
            Self::Token(tkn) => tkn.kind() == TomlKind::Whitespace,
        }
    }
    pub fn kind(&self) -> TomlKind {
        match self {
            Self::Node(node) => node.kind(),
            Self::Token(tkn) => tkn.kind(),
        }
    }
    pub fn as_node(&self) -> Option<&TomlNode> {
        match self {
            Element::Node(node) => Some(node),
            Element::Token(_token) => None,
        }
    }
    pub fn as_token(&self) -> Option<&TomlToken> {
        match self {
            Element::Token(token) => Some(token),
            Element::Node(_node) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TomlNode {
    pub(crate) kind: TomlKind,
    pub(crate) text: SmolStr,
}

impl TomlNode {
    /// Returns self wrapped in `Element`, this clones self.
    pub fn to_ele(&self) -> Element {
        Element::Node(self.clone())
    }
    pub fn kind(&self) -> TomlKind {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TomlToken {
    pub(crate) kind: TomlKind,
    pub(crate) text: SmolStr,
}

impl TomlToken {
    /// Returns self wrapped in `Element`, this clones self.
    pub fn to_ele(&self) -> Element {
        Element::Token(self.clone())
    }

    pub fn kind(&self) -> TomlKind {
        self.kind
    }
    pub fn text(&self) -> &SmolStr {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum TomlKind {
    //
    // TOKENS
    // these are considered tokens from
    // here down
    /// the text of a comment.
    CommentText = 0,
    /// A signed 64 bit number.
    Integer,
    /// True or false.
    Bool,
    /// The token when a key is not surrounded by quotes.
    Ident,

    /// Single quote.
    SingleQuote,
    /// Double quote, used for keys and strings.
    DoubleQuote,
    /// Triple quote, used for literal strings.
    TripleQuote,

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
    /// End of file token.
    EoF,

    // NODES
    // these are nodes
    //
    Table,
    /// A table heading surounded by brackets.
    Heading,
    /// A segmented `Heading` Ident.
    SegIdent,
    /// An inline table where the key is the "heading" and
    /// key value pairs inside of curly braces.
    InlineTable,
    /// A key and a value, any other valid toml type.
    KeyValue,
    /// A key either `Ident` or double quoted.
    Key,
    /// Any valid toml type after a key.
    Value,
    /// A toml array.
    Array,
    ///
    ArrayItem,
    /// Toml date
    /// TODO this is one of with offset, without, local,
    /// time, date and datetime.
    Date,
    /// A toml table consisting of a heading and key
    /// value pairs.
    /// An signed 64 bit EEE 754-2008 "binary64" number.
    Float,
    /// One of three string types, literal single quote,
    /// normal double quote and literal triple double quote.
    /// (like python doc comments)
    Str,
    /// A comment in the toml file, a `Hash` token followed by `CommentText`.
    Comment,
    /// the "empty" root node representing a whole file.
    Root,
}

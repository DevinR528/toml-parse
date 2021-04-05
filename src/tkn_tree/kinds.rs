use rowan::SmolStr;

/// An enum representing either a `Node` or a `Token`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Element {
    #[allow(dead_code)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TomlNode {
    pub(crate) kind: TomlKind,
    pub(crate) text: SmolStr,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TomlToken {
    pub(crate) kind: TomlKind,
    pub(crate) text: SmolStr,
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

    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `=`
    Equal,
    /// `#`
    Hash,
    /// `,`
    Dot,
    /// `,`
    Comma,
    /// `:`
    Colon,

    /// opening brace `{`.
    OpenCurly,
    /// closing brace `}`.
    CloseCurly,
    /// opening brace `[`.
    OpenBrace,
    /// closing brace `]`.
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
    /// A table heading surrounded by brackets.
    Heading,
    /// An array of tables heading
    ArrayHeading,
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
    /// An item within a toml array.
    ArrayItem,
    /// Toml date
    /// TODO this is one of with offset, without, local,
    /// time, date and datetime.
    Date,
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

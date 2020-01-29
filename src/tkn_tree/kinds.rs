use smol_str::SmolStr;
use std::ops::Range;
use std::fmt;
use std::iter;
use std::rc::Rc;

type TextRange = Range<usize>;

/// An enum representing either a `Node` or a `Token`.
#[derive(Clone, Debug, PartialEq)]
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
    fn first_child_or_token(&self) -> Option<Element> {
        match self {
            Element::Node(node) => node.children.get(0).map(|n| n.clone()),
            Element::Token(tkn) => Some(tkn.to_ele()),
        }
    }
    pub fn next_sibling_or_token(&self) -> Option<Element> {
        match self {
            Element::Node(node) => node.next_sibling_or_token(),
            Element::Token(tkn) => tkn.next_sibling_or_token(),
        }
    }
    pub fn parent(&self) -> Option<Element> {
        match self {
            Element::Node(node) => node.parent().map(|n| Element::Node(*n.clone())),
            Element::Token(tkn) => tkn.parent().map(|n| Element::Node(*n.clone())),
        }
    }
}

pub enum DebugEvent {
    Further(Element),
    Out(Element),
}

#[derive(Clone, PartialEq)]
pub struct TomlNode {
    pub(crate) kind: TomlKind,
    pub(crate) parent: Rc<TomlNode>,
    pub(crate) text: SmolStr,
    pub(crate) children: Vec<Element>,
    pub(crate) range: TextRange,
}

impl TomlNode {
    fn first_child_or_token(&self) -> Option<Element> {
        self.children.get(0).map(|n| n.clone())
    }
    pub fn next_sibling_or_token(&self) -> Option<Element> {
        if let Some(idx) = self.parent.children.iter().position(|n| n == self) {
            self.parent.children.get(idx + 1).map(|n| n.clone())
        } else {
            None
        }
    }
    pub fn walk_with_tokens(&self) -> impl Iterator<Item = DebugEvent> {
        let start: Element = self.to_ele();
        iter::successors(Some(DebugEvent::Further(start.clone())), move |pos| {
            let next = match pos {
                DebugEvent::Further(el) => match el {
                    Element::Node(node) => match node.first_child_or_token() {
                        Some(child) => DebugEvent::Further(child),
                        None => DebugEvent::Out(node.to_ele()),
                    },
                    Element::Token(token) => DebugEvent::Out(token.to_ele()),
                },
                DebugEvent::Out(el) => {
                    if el == &start {
                        return None;
                    }
                    match el.next_sibling_or_token() {
                        Some(sibling) => DebugEvent::Further(sibling),
                        None => DebugEvent::Out(el.parent().unwrap().into()),
                    }
                }
            };
            Some(next)
        })
        // let mut collected = Vec::default();
        // for ele in self.children.iter() {
        //     match ele {
        //         Element::Node(node) => {
        //             collected.push(DebugEvent::Further(ele));
        //             collected.extend(node.walk_with_tokens());
        //         },
        //         Element::Token(_) => collected.push(DebugEvent::Same(ele)),
        //     }
        // }
        // collected.push(DebugEvent::Out);
        // collected
    }
    pub fn parent(&self) -> Option<Rc<TomlNode>> {
        match self.kind() {
            TomlKind::Root => None,
            _ => Some(Rc::clone(&self.parent)),
        }
    }
    pub fn to_ele(&self) -> Element {
        Element::Node(self.clone())
    }

    pub fn kind(&self) -> TomlKind {
        self.kind
    }

    pub fn start(&self) -> usize {
        self.range.start
    }
    pub fn end(&self) -> usize {
        self.range.end
    }
    pub fn text_range(&self) -> &TextRange {
        &self.range
    }
}

impl fmt::Debug for TomlNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut indent = 0;
            for ele in self.walk_with_tokens() {
                for _ in 0..indent {
                    write!(f, "  ")?;
                }
                match ele {
                    DebugEvent::Further(el) => {
                        match el {
                            Element::Node(n) => writeln!(f, "{:?}", n)?,
                            Element::Token(n) => writeln!(f, "{:?}", n)?,
                        }
                        indent += 1;
                    },
                    DebugEvent::Out(_) => indent -= 1,
                }
            }
            Ok(())
        } else {
            write!(f, "{:?}@{:?}", self.kind(), self.text_range())
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TomlToken {
    pub(crate) kind: TomlKind,
    pub(crate) parent: Rc<TomlNode>,
    pub(crate) text: SmolStr,
    pub(crate) range: TextRange,
}

impl fmt::Debug for TomlToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind(), self.text_range())?;
        if self.text().len() < 25 {
            return write!(f, " {:?}", self.text());
        }
        let text = self.text().as_str();
        for idx in 21..25 {
            if text.is_char_boundary(idx) {
                let text = format!("{} ...", &text[..idx]);
                return write!(f, " {:?}", text);
            }
        }
        unreachable!()
    }
}

impl TomlToken {
    // pub fn new(text: &str, range: TextRange) -> Self {

    // }
    pub fn next_sibling_or_token(&self) -> Option<Element> {
        if let Some(idx) = self.parent.children.iter().position(|n| n == self) {
            self.parent.children.get(idx + 1).map(|n| n.clone())
        } else {
            None
        }
    }

    pub fn parent(&self) -> Option<Rc<TomlNode>> {
        match self.kind() {
            TomlKind::Root => None,
            _ => Some(Rc::clone(&self.parent)),
        }
    }

    pub fn to_ele(&self) -> Element {
        Element::Token(self.clone())
    }

    pub fn kind(&self) -> TomlKind {
        self.kind
    }
    pub fn text(&self) -> &SmolStr {
        &self.text
    }
    pub fn start(&self) -> usize {
        self.range.start
    }
    pub fn end(&self) -> usize {
        self.range.end
    }
    pub fn text_range(&self) -> &TextRange {
        &self.range
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomlKind {
    // NODES
    // these are nodes
    /// the "empty" root node representing a whole file.
    Root,
    /// A toml array.
    Array,
    ///
    ArrayItem,
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
    /// An signed 64 bit EEE 754-2008 "binary64" number.
    Float,
    /// One of three string types, literal single quote,
    /// normal double quote and literal triple double quote.
    /// (like python doc comments)
    Str,
    /// A key either `Ident` or double quoted.
    Key,
    /// A comment in the toml file, a `Hash` token followed by `CommentText`.
    Comment,

    //
    // TOKENS
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
    /// True or false.
    Bool,
    /// The token when a key is not surrounded by quotes.
    Ident,

    /// Single quote.
    SingleQuote,
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
    /// End of file token.
    EoF,
}

pub enum Token {}

impl Element {
    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::Node(_) => false,
            Self::Token(tkn) => tkn.kind() == TomlKind::Whitespace,
        }
    }
}

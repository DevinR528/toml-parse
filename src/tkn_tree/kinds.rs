use smol_str::SmolStr;
use std::ops::Range;
use std::fmt;
use std::iter;
use std::collections::{HashMap, HashSet, BTreeSet, BTreeMap};

use std::sync::Mutex;
use std::cell::UnsafeCell;
use std::rc::Rc;
use lazy_static::lazy_static;

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
    fn first_child_or_token(&self) -> Option<Element> {
        match self {
            Element::Node(node) => node.children.get(0).map(|n| n.clone()),
            Element::Token(tkn) => Some(tkn.to_ele()),
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
    pub fn ancestors<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a TomlNode> + 'a)> {
        match self {
            Element::Node(node) => Box::new(node.ancestors()),
            Element::Token(tkn) => Box::new(tkn.ancestors()),
        }
    }
    pub fn walk_with_tokens(&self) -> Vec<Element> {
        self.as_node().unwrap().walk_with_tokens()
    }
}


// thread_local! { static PARENTS: UnsafeCell<Vec<&'static mut TomlNode>> = UnsafeCell::new(Vec::new()); }

static mut PAR: Vec<&'static mut TomlNode> = Vec::new();


/// TODO make this and `StaticRef` a single struct with a safe
/// api.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StaticRef;

impl StaticRef {

    pub fn show(&self) -> &dyn fmt::Debug {
        unsafe { &PAR as &dyn fmt::Debug }
    }

    pub fn len(&self) -> usize {
        unsafe { PAR.len() }
    }

    pub fn push(&self, node: &'static mut TomlNode) -> usize {
        unsafe { 
            if let Some(idx) = PAR.iter().position(|n| *n == node) {
                return idx;
            } else {
                PAR.push(node);
                return self.len() - 1
            }
        }
    }

    pub fn get(&self, idx: usize) -> &TomlNode {
        unsafe { PAR.get(idx).unwrap() }
    }
    /// # Safety
    /// 
    /// The mutable reference returned is from a static vec of `TomlNode`
    /// and is only used mutably when the `TomlNode` is created, and only ever
    /// read after.
    /// 
    #[allow(clippy::mut_from_ref)]
    fn get_mut(&self, idx: usize) -> &mut TomlNode {
        unsafe { PAR.get_mut(idx).unwrap() }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Ancestor {
    Parent { s_ref: StaticRef, idx: usize },
    Root,
}

impl fmt::Debug for Ancestor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ancestor::")?;
        match self {
            Ancestor::Root { .. } => write!(f, "Root"),
            Ancestor::Parent { s_ref, idx } => {
                write!(f, "Parent( {:?} )", s_ref.get(*idx).kind())
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TomlNode {
    pub(crate) kind: TomlKind,
    pub(crate) parent: Ancestor,
    pub(crate) text: SmolStr,
    pub(crate) children: Vec<Element>,
    pub(crate) range: TextRange,
}

// impl fmt::Debug for TomlNode {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if f.alternate() {
//             f.debug_struct("Node")
//             .field("kind", &self.kind())
//             .field("parent", &self.parent)
//             .field("children", &self.children)
//             .field("range", &self.range)
//             .finish()
//         } else {
//             write!(f, "{:?}@{:?}", self.kind(), self.text_range())
//         }
//     }
// }

impl TomlNode {
    pub fn new(kind: TomlKind, text: SmolStr, range: TextRange, children: Vec<Element>) -> TomlNode {
        let s_ref = StaticRef;
        let root = Box::new(TomlNode {
            kind,
            parent: Ancestor::Root,
            text,
            range,
            children,
        });
        
        let idx = s_ref.push(Box::leak(root));
        let root = s_ref.get_mut(idx);

        for ele in root.children.iter_mut() {
            match ele {
                Element::Node(node) => node.parent = Ancestor::Parent { s_ref, idx },
                Element::Token(tkn) => tkn.parent = Ancestor::Parent { s_ref, idx },
            }
        }
        unsafe { std::ptr::read(root as *const _) }
    }
    
    pub fn ancestors(&self) -> impl Iterator<Item = &TomlNode> {
        iter::successors(Some(self), |node| {
            if let Ancestor::Parent { s_ref, idx } = node.parent() {
                let parent = s_ref.get(*idx);
                Some(parent)
            } else {
                let s = StaticRef;
                unsafe { PAR.get(s.len()).map(|n| &**n) }
            }
        })
    }

    /// TODO try this as a generator?
    fn walk_with_tokens_debug(&self) -> Vec<DebugEvent> {
        let start: Element = self.to_ele();
        let mut collected = vec![DebugEvent::Further(start)];
        for ele in self.children.iter() {
            match ele {
                Element::Node(node) => {
                    collected.extend(node.walk_with_tokens_debug());
                },
                Element::Token(_) => collected.push(DebugEvent::Same(ele.clone())),
            }
        }
        collected.push(DebugEvent::Out);
        collected
    }
    /// TODO try this as a generator?
    pub fn walk_with_tokens(&self) -> Vec<Element> {
        let start: Element = self.to_ele();
        let mut collected = vec![start];
        for ele in self.children.iter() {
            match ele {
                Element::Node(node) => {
                    collected.extend(node.walk_with_tokens());
                },
                Element::Token(_) => collected.push(ele.clone()),
            }
        }
        collected
    }
    // pub fn parent(&self) -> Option<Rc<TomlNode>> {
    //     match self.kind() {
    //         TomlKind::Root => None,
    //         _ => Some(Rc::clone(&self.parent)),
    //     }
    // }
    /// Returns self wrapped in `Element`, this clones self.
    pub fn to_ele(&self) -> Element {
        Element::Node(self.clone())
    }

    pub fn parent(&self) -> &Ancestor {
        &self.parent
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

pub enum DebugEvent {
    Further(Element),
    Same(Element),
    Out,
}

impl fmt::Debug for TomlNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut indent = 0;
            for ele in self.walk_with_tokens_debug() {
                
                match ele {
                    DebugEvent::Further(el) => {
                        for _ in 0..indent {
                            write!(f, "  ")?;
                        }
                        match &el {
                            Element::Node(n) => writeln!(f, "{:?}", n)?,
                            Element::Token(n) => writeln!(f, "{:?}", n)?,
                        }
                        println!("{:?}", el.ancestors().collect::<Vec<_>>());
                        indent += 1;
                    },
                    DebugEvent::Same(el) => {
                        for _ in 0..indent {
                            write!(f, "  ")?;
                        }
                        match &el {
                            Element::Node(n) => writeln!(f, "{:?}", n)?,
                            Element::Token(n) => writeln!(f, "{:?}", n)?,
                        }
                        println!("{:?}", el.ancestors().collect::<Vec<_>>());
                    },
                    DebugEvent::Out => indent -= 1,
                }
            }
            assert_eq!(indent, 0);
            let s = StaticRef;
            println!("{:?}", s.len());
            Ok(())
        } else {
            write!(f, "{:?}@{:?}", self.kind(), self.text_range())
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TomlToken {
    pub(crate) kind: TomlKind,
    pub(crate) parent: Ancestor,
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
    // pub fn next_sibling_or_token(&self) -> Option<Element> {
    //     if let Some(idx) = self.parent.children.iter().position(|n| n == self) {
    //         self.parent.children.get(idx + 1).map(|n| n.clone())
    //     } else {
    //         None
    //     }
    // }

    // pub fn parent(&self) -> Option<Rc<TomlNode>> {
    //     match self.kind() {
    //         TomlKind::Root => None,
    //         _ => Some(Rc::clone(&self.parent)),
    //     }
    // }

    pub fn to_ele(&self) -> Element {
        Element::Token(self.clone())
    }

    pub fn parent(&self) -> &Ancestor {
        &self.parent
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

    pub fn ancestors(&self) -> impl Iterator<Item = &TomlNode> {
        let start = if let Ancestor::Parent { s_ref, idx } = self.parent() {
            Some(s_ref.get(*idx))
        } else {
            None
        };
        iter::successors(start, |node| {
            if let Ancestor::Parent { s_ref, idx } = node.parent() {
                let parent = s_ref.get(*idx);
                Some(parent)
            } else {
                let s = StaticRef;
                unsafe { PAR.get(s.len()).map(|n| &**n) }
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
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

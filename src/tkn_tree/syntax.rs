use rowan::{GreenNode, GreenNodeBuilder};

use super::err::TomlResult;
use super::kinds::TomlKind::{self, *};
use super::parse_tkns::Tokenizer;
use super::walk::{walk, walk_tokens};

pub type SyntaxNode = rowan::SyntaxNode<TomlLang>;
pub type SyntaxToken = rowan::SyntaxToken<TomlLang>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

/// The main trait to go from untyped `SyntaxNode`  to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode {
    fn can_cast(kind: TomlKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: TomlKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

pub trait SyntaxNodeExtTrait {
    /// walks tokens collecting each tokens text into a final String.
    fn token_text(&self) -> String;
    /// `rowan::SyntaxNode` by default only compares pointer equality
    /// this method addition allows comparison of every token, the same
    /// file parsed multiple times will return true, with pointer eq
    /// this would be false.
    fn deep_eq(&self, other: &Self) -> bool;
}

impl From<TomlKind> for rowan::SyntaxKind {
    fn from(kind: TomlKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TomlLang;
impl rowan::Language for TomlLang {
    type Kind = TomlKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= Root as u16);
        unsafe { std::mem::transmute::<u16, TomlKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

impl SyntaxNodeExtTrait for SyntaxNode {
    fn token_text(&self) -> String {
        walk_tokens(self).fold(String::default(), |mut s, tkn| {
            s.push_str(tkn.text());
            s
        })
    }

    fn deep_eq(&self, other: &Self) -> bool {
        for (a, b) in walk(self).zip(walk(other)) {
            match (&a, &b) {
                (SyntaxElement::Node(n1), SyntaxElement::Node(n2)) => {
                    if n1.token_text() != n2.token_text() {
                        return false;
                    }
                }
                (SyntaxElement::Token(t1), SyntaxElement::Token(t2)) => {
                    if t1.text() != t2.text() {
                        return false;
                    }
                }
                (_, _) => return false,
            }
            if a.kind() != b.kind() {
                return false;
            }
        }
        true
    }
}

pub struct ParsedToml {
    green: rowan::GreenNode,
}

impl ParsedToml {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

pub struct Parser {
    /// the in-progress tree.
    pub(crate) builder: GreenNodeBuilder<'static>,
}

impl Default for Parser {
    fn default() -> Self {
        Parser::new()
    }
}

impl Parser {
    pub fn new() -> Parser {
        Self {
            builder: GreenNodeBuilder::new(),
        }
    }
    pub fn parse(self) -> TomlResult<ParsedToml> {
        let green: GreenNode = self.builder.finish();
        // Construct a `SyntaxNode` from `GreenNode`,
        // Since we only want valid toml errors cause a bubble up
        // failure, not passed along in the tree as they can be.
        Ok(ParsedToml { green })
    }
}

/// Parses the input into a [`Result<ParsedToml>`][ParsedToml].
///
/// This contains a [`GreenNode`][rowan::GreenNode] and
/// by calling `.syntax()` on `ParsedToml` you get the `TomlKind::Root`
/// [`SyntaxNode`][rowan::SyntaxNode].
///
/// # Examples
/// ```
/// use toml_parse::{parse_it, TomlKind};
///
/// let toml =
/// "[valid]
/// toml = \"stuff\"
/// ";
///
/// let root_node = parse_it(toml).unwrap().syntax();
/// assert_eq!(root_node.first_child().unwrap().kind(), TomlKind::Table)
/// ```
pub fn parse_it(input: &str) -> TomlResult<ParsedToml> {
    let parse_builder = Parser::new();
    let parsed = Tokenizer::parse(input, parse_builder)?;
    parsed.parse()
}

use rowan::SmolStr;

use super::kinds::{TomlKind::{self, *}, };
use super::err::{TomlErrorKind, ParseTomlError, TomlResult};
impl From<TomlKind> for rowan::SyntaxKind {
    fn from(kind: TomlKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Lang {}
impl rowan::Language for Lang {
    type Kind = TomlKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= Root as u16);
        unsafe { std::mem::transmute::<u16, TomlKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

type SyntaxNode = rowan::SyntaxNode<Lang>;
#[allow(unused)]
type SyntaxToken = rowan::SyntaxToken<Lang>;
#[allow(unused)]
type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

pub struct ParseResult {
    green: rowan::GreenNode,
    errs: Vec<ParseTomlError>,
}

impl Parse {
    fn syntax(&self) -> Root {
        Root::cast(SyntaxNode::new_root(self.green_node.clone())).unwrap()
    }
}

pub struct Parser {
    /// input tokens, including whitespace,
    /// in *reverse* order.
    tokens: Vec<(SyntaxKind, SmolStr)>,
    /// the in-progress tree.
    builder: GreenNodeBuilder<'static>,
    /// the list of syntax errors we've accumulated
    /// so far.
    errors: Vec<String>,
}



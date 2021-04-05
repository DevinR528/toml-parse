use std::marker::PhantomData;

use rowan::SyntaxNodeChildren;

use crate::tkn_tree::{
    AstNode, AstToken, SyntaxElement, SyntaxNode, SyntaxNodeExtTrait, SyntaxToken, TomlKind,
    TomlLang,
};

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren<TomlLang>,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
    AstChildren::new(parent)
}

fn token(parent: &SyntaxNode, kind: TomlKind) -> Option<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == kind)
}

pub struct KeyValue {
    syntax: SyntaxNode,
}

impl AstNode for KeyValue {
    fn can_cast(kind: TomlKind) -> bool {
        kind == TomlKind::KeyValue
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl KeyValue {
    pub fn name(&self) -> Option<String> {
        token(&self.syntax, TomlKind::Ident).map(|t| t.to_string())
    }
    pub fn value(&self) -> Option<Value> {
        child(&self.syntax)
    }
}

pub enum Value {
    InlineTable(InlineTable),
    Array(Array),
    Str(Str),
    Date(Date),
    Float(Float),
}

impl AstNode for Value {
    fn can_cast(kind: TomlKind) -> bool {
        matches!(
            kind,
            TomlKind::InlineTable
                | TomlKind::Array
                | TomlKind::Str
                | TomlKind::Date
                | TomlKind::Float
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        Some(match syntax.kind() {
            TomlKind::InlineTable => Value::InlineTable(InlineTable { syntax }),
            TomlKind::Array => Value::Array(Array { syntax }),
            TomlKind::Str => Value::Str(Str { syntax }),
            TomlKind::Date => Value::Date(Date { syntax }),
            TomlKind::Float => Value::Float(Float { syntax }),
            _ => return None,
        })
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Value::InlineTable(it) => it.syntax(),
            Value::Array(it) => it.syntax(),
            Value::Str(it) => it.syntax(),
            Value::Date(it) => it.syntax(),
            Value::Float(it) => it.syntax(),
        }
    }
}

pub struct InlineTable {
    syntax: SyntaxNode,
}

impl AstNode for InlineTable {
    fn can_cast(kind: TomlKind) -> bool {
        kind == TomlKind::InlineTable
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
pub struct Array {
    syntax: SyntaxNode,
}

impl AstNode for Array {
    fn can_cast(kind: TomlKind) -> bool {
        kind == TomlKind::Array
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
pub struct Str {
    syntax: SyntaxNode,
}
impl AstNode for Str {
    fn can_cast(kind: TomlKind) -> bool {
        kind == TomlKind::Str
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
pub struct Date {
    syntax: SyntaxNode,
}
impl AstNode for Date {
    fn can_cast(kind: TomlKind) -> bool {
        kind == TomlKind::Date
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
pub struct Float {
    syntax: SyntaxNode,
}
impl AstNode for Float {
    fn can_cast(kind: TomlKind) -> bool {
        kind == TomlKind::Float
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

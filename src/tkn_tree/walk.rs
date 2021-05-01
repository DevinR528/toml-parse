use rowan::WalkEvent;

use super::{
    syntax::{SyntaxElement, SyntaxNode, SyntaxToken},
    TomlKind,
};

pub fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element),
        WalkEvent::Leave(_) => None,
    })
}

pub fn walk_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    walk(node).filter_map(|element| match element {
        SyntaxElement::Token(token) => Some(token),
        _ => None,
    })
}
pub fn walk_tokens_non_ws(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    walk(node).filter_map(|element| match element {
        SyntaxElement::Token(token) if token.kind() != TomlKind::Whitespace => {
            Some(token)
        }
        _ => None,
    })
}

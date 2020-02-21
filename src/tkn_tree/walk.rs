use std::iter::successors;

use rowan::WalkEvent;

use super::syntax::{SyntaxElement, SyntaxNode, SyntaxToken};
use super::TomlKind;

pub fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element),
        WalkEvent::Leave(_) => None,
    })
}
pub fn walk_non_whitespace(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element).filter(|it| it.kind() != TomlKind::Whitespace),
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
        SyntaxElement::Token(token) if token.kind() != TomlKind::Whitespace => Some(token),
        _ => None,
    })
}
pub fn walk_nodes(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> {
    walk(node).filter_map(|element| match element {
        SyntaxElement::Node(node) => Some(node),
        _ => None,
    })
}
pub fn has_newline(node: &SyntaxNode) -> bool {
    walk_tokens(node).any(|it| it.text().contains('\n'))
}
pub fn prev_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.prev_sibling_or_token(), |it| {
        it.prev_sibling_or_token()
    })
    .find(|it| it.kind() != TomlKind::Whitespace)
}
pub fn prev_siblings(element: &SyntaxElement) -> impl Iterator<Item = SyntaxElement> {
    successors(element.prev_sibling_or_token(), |it| {
        it.prev_sibling_or_token()
    })
}
pub fn next_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.next_sibling_or_token(), |it| {
        it.next_sibling_or_token()
    })
    .find(|it| it.kind() != TomlKind::Whitespace)
}

pub fn next_siblings(element: &SyntaxElement) -> impl Iterator<Item = SyntaxElement> {
    successors(element.next_sibling_or_token(), |it| {
        it.next_sibling_or_token()
    })
}

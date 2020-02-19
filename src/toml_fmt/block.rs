use super::tkn_tree::{
    walk::{
        next_siblings, prev_non_whitespace_sibling, prev_siblings, walk_nodes, walk_non_whitespace,
        walk_tokens, walk,
    },
    SyntaxNodeExtTrait, SyntaxElement, SyntaxNode, SyntaxToken, TomlKind,
};

use super::ws::WhiteSpace;

pub struct Block {
    ele: SyntaxElement,
    whitespace: WhiteSpace,
}

impl Block {
    pub fn new(ele: SyntaxElement) -> Block {
        let whitespace = WhiteSpace::new(&ele);
        Self {
            ele,
            whitespace,
        }
    }
}

use super::tkn_tree::{
    walk::{
        next_siblings, prev_non_whitespace_sibling, prev_siblings, walk_nodes, walk_non_whitespace,
        walk_tokens, walk,
    },
    SyntaxNodeExtTrait, SyntaxElement, SyntaxNode, SyntaxToken, TomlKind,
};

use super::ws::WhiteSpace;

pub struct Block {
    tkn: SyntaxToken,
    whitespace: WhiteSpace,
}

impl Block {
    pub fn new(tkn: SyntaxToken) -> Block {
        let whitespace = WhiteSpace::new(&tkn);
        Self {
            tkn,
            whitespace,
        }
    }
}

pub(self) use super::tkn_tree::{
    self,
    walk::{
        next_siblings, prev_non_whitespace_sibling, prev_siblings, walk_nodes, walk_non_whitespace,
        walk_tokens, walk,
    },
    SyntaxNodeExtTrait, SyntaxElement, SyntaxNode, SyntaxToken, TomlKind,
};

mod block;
mod ws;

use block::Block;

pub struct Formatter {
    blocks: Vec<Block>,
}

impl Formatter {
    pub fn new(root: &SyntaxNode) -> Formatter {
        let blocks = walk_tokens(root).map(Block::new).collect();
        Self { blocks, }
    }

    pub fn format(&mut self) {
        
    }
}

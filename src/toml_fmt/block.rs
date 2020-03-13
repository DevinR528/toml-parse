use std::cell::Cell;
use std::fmt;

use rowan::Direction;

use super::tkn_tree::{SyntaxElement, SyntaxToken, TomlKind};
use super::ws::WhiteSpace;

#[derive(Debug, Clone)]
pub struct Block {
    tkn: SyntaxToken,
    pub(crate) whitespace: Cell<WhiteSpace>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.whitespace.get().space_before)?;
        write!(f, "{}", self.tkn.text())
    }
}

impl Block {
    pub fn new(tkn: SyntaxToken) -> Block {
        let whitespace = Cell::new(WhiteSpace::new(&tkn));
        Self { tkn, whitespace }
    }

    pub fn kind(&self) -> TomlKind {
        self.tkn.kind()
    }

    pub fn token(&self) -> &SyntaxToken {
        &self.tkn
    }

    pub fn whitespace(&self) -> WhiteSpace {
        self.whitespace.get()
    }

    pub fn parents_contain(&self, pat: &str) -> bool {
        let newline = |node: SyntaxElement| -> bool {
            match node {
                SyntaxElement::Token(t) => t.text().contains(pat),
                _ => false,
            }
        };
        if self
            .tkn
            .parent()
            .siblings_with_tokens(Direction::Next)
            .any(newline)
        {
            return true;
        }
        self.tkn
            .parent()
            .siblings_with_tokens(Direction::Next)
            .any(newline)
    }
}

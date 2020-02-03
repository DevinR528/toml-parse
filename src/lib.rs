mod common;
// mod structure;
mod struc;
mod tkn_tree;

// pub use structure::{Heading, KvPairs, Table, Toml, Value};
pub use tkn_tree::{
    parse_it, ParsedToml, Parser, SyntaxElement, SyntaxNode, SyntaxToken, Tokenizer,
};

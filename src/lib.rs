#![allow(dead_code)]
#![allow(unused)]
#![allow(unused_mut)]

mod common;
// mod structure;
mod struc;
mod tkn_tree;

pub use struc::{Heading, InTable, KvPair, Table, Toml, Value};
pub use tkn_tree::{
    parse_it, ParsedToml, Parser, SyntaxElement, SyntaxNode, SyntaxToken, Tokenizer,
};
pub use common::munch::Muncher;

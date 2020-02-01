use rowan::{GreenNode, GreenNodeBuilder, SmolStr};

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::kinds::TomlKind::{self, *};
use super::parse_tkns::Tokenizer;

impl From<TomlKind> for rowan::SyntaxKind {
    fn from(kind: TomlKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TomlLang;
impl rowan::Language for TomlLang {
    type Kind = TomlKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= Root as u16);
        unsafe { std::mem::transmute::<u16, TomlKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<TomlLang>;
pub type SyntaxToken = rowan::SyntaxToken<TomlLang>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

pub struct ParseToml {
    green: rowan::GreenNode,
}

impl ParseToml {
    fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

pub struct Parser {
    /// the in-progress tree.
    pub(crate) builder: GreenNodeBuilder<'static>,
}

impl Parser {
    fn parse(self) -> TomlResult<ParseToml> {
        let green: GreenNode = self.builder.finish();
        // Construct a `SyntaxNode` from `GreenNode`,
        // using errors as the root data.
        Ok(ParseToml { green: green })
    }
}

pub fn parse_it(input: &str) -> TomlResult<ParseToml> {
    let parser = Parser {
        builder: GreenNodeBuilder::new(),
    };
    let parsed = Tokenizer::parse(input, parser)?;
    parsed.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    #[test]
    fn token_file() {
        // ftop.toml is 7 items long
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let parsed = parse_it(&input).expect("parse failed");
        // assert_eq!(parsed.len(), 7);
        println!("{:#?}", parsed.syntax());
    }

    #[test]
    fn parents() {
        let file = "[table]\n# hello there";
        let parsed = parse_it(file).expect("parse failed");

        println!("{:#?}", parsed.syntax().first_token());

        // for ele in parsed.walk_with_tokens() {
        //     println!("{:?}", ele);
        //     println!("{:?}", ele.ancestors().collect::<Vec<_>>())
        // }
    }

    #[test]
    fn all_tokens() {
        let file = r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;
        let parsed = parse_it(file).expect("parse failed");

        println!("{:#?}", parsed.syntax());

        // for ele in parsed.walk_with_tokens() {
        //     println!("{:?}", ele);
        //     println!("{:?}", ele.ancestors().collect::<Vec<_>>())
        // }
    }
}

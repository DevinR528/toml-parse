use smol_str::SmolStr;

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::kinds::{Element, TomlNode, TomlToken};
use super::munch::Muncher;

impl Element {
    fn root(muncher: &mut Muncher) -> Element {
        let text = SmolStr::new(muncher.text());
    }
}

pub struct Tokenizer {
    ast: Vec<Element>,
}

impl Tokenizer {
    pub fn parse(input: &str) -> TomlResult<Tokenizer> {
        let mut muncher = Muncher::new(input);
        let mut parsed = Element::root(&mut muncher);
        loop {
            let value = Value::parse(muncher)?;
            if value == Value::Eof {
                break;
            };
            parsed.push(value);
        }
        Ok(Tokenizer { ast: parsed })
    }
}

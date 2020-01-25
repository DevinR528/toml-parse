use chrono::{Date, Datelike, DateTime, NaiveDateTime, NaiveDate, NaiveTime};

mod err;
mod parse;
mod token;

use err::{TomlErrorKind, ParseTomlError, TomlResult};
use parse::Parse;
use token::{cmp_tokens, Muncher};

const EOL: &[char] = &[ '\n', '\r' ];
const BOOL: &[char] = &[ 't', 'f' ];
const KEY_END: &[char] = &[ '\n', '\r', ' ', ',', '='];
const SPACE_EQ: &[char] = &[ ' ', '=' ];

#[derive(Debug, Clone)]
pub struct Heading {
    header: String,
    seg: Vec<String>,
}

impl Parse for Heading {
    type Item = Heading;
    fn parse(muncher: &mut Muncher) -> TomlResult<Heading> {
        let mut header = String::default();
        let mut seg = Vec::default();
        for ch in muncher.eat_until(|c| c == &']') {
            if ch == '.' {
                seg.push(header.clone());
            }
            header.push(ch);
        }
        // remove last closing brace;
        assert!(muncher.eat_close_brc());
        println!("{:?}", muncher);
        assert!(muncher.eat_eol());

        Ok(Self {
            header,
            seg,
        })
    }
}

#[derive(Debug, Clone)]
struct KvPairs {
    key: Option<String>,
    val: Value,
}

impl KvPairs {
    fn parse_while(muncher: &mut Muncher) -> TomlResult<Option<(String, Value)>> {
        let mut key = muncher.eat_until(|c| cmp_tokens(c, KEY_END)).collect::<String>();
        if key.starts_with('"') {
            key.remove(0);
        }
        let val: TomlResult<Value>;
        let fork = muncher.fork();
        if fork.seek(3).map(|s| s.contains('=')) == Some(true)  {
            // eats whitespace if found
            muncher.eat_ws();
            // eats eq and optionally whitespace after.
            muncher.eat_eq();
            val = match muncher.peek() {
                Some('"') => Value::parse_str(muncher),
                None => Ok(Value::Eof),
                _ => {
                    let msg = "invalid token in key value pairs";
                    let tkn = if let Some(peek) = muncher.peek() {
                        format!("{:?}", peek)
                    } else {
                        "no token".into()
                    };
                    Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(tkn)))
                },
            }
        } else {
            println!("{:#?}", fork);
            let msg = "invalid token in key value pairs";
            let tkn = if let Some(peek) = muncher.peek() {
                format!("{:?}", peek)
            } else {
                "no token".into()
            };
            val = Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(tkn)));
        }

        if let Ok(Value::Eof) = val {
            return Ok(None);
        }
        println!("{:?} {:?}", key, val);
        Ok(Some((key, val?)))
    }
}

impl Parse for KvPairs {
    type Item = Vec<KvPairs>;
    fn parse(muncher: &mut Muncher) -> Result<Vec<KvPairs>, ParseTomlError> {
        let mut pairs = Vec::default();
        loop {
            let pair = KvPairs::parse_while(muncher)?;
            if let Some((key, val)) = pair {
                let key = if key.is_empty() {
                    None
                } else {
                    Some(key)
                };
                pairs.push(Self { key, val, });
            } else {
                break;
            }
        }
        Ok(pairs)
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    header: Heading,
    pairs: Vec<KvPairs>,
}

impl Parse for Table {
    type Item = Table;
    fn parse(muncher: &mut Muncher) -> Result<Table, ParseTomlError> {
        let _open_brace = muncher.eat();
        let header = Heading::parse(muncher)?;
        println!("{:#?}", header);
        let _close_brace = muncher.eat();
        let pairs = KvPairs::parse(muncher)?;
        Ok(Self {
            header,
            pairs,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(isize),
    Float(f64),
    StrLit(String),
    Date(NaiveDateTime),
    Array(Vec<Value>),
    Table(Table),
    Comment(String),
    Eof,
}

impl Value {
    fn parse_str(muncher: &mut Muncher) -> TomlResult<Self> {
        println!("parse str");
        let mut pair = 0;
        let mut s = muncher.eat_until(|c| {
                if c == &'"'{
                    pair += 1;
                    pair == 2
                } else {
                    false
                }
            })
            .collect::<String>();

        if s.starts_with('"') {
            s.remove(0);
        }
        assert!(muncher.eat_quote());
        let end = muncher.peek();
        if end == Some(&'\n') || end == Some(&'\r') {
            assert!(muncher.eat_eol());
            Ok(Self::StrLit(s))
        } else {
            let msg = "invalid token in value";
            let tkn = if let Some(peek) = end {
                format!("{:?}", peek)
            } else {
                "no token".into()
            };
            Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(tkn)))
        }
    }

    fn parse_array(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher.eat_until(|c| c == &']').collect::<String>();
        let _new_line = muncher.eat();
        if s == "true" {
            Ok(Value::Bool(true))
        } else if s == "false" {
            Ok(Value::Bool(false))

        } else {
            let msg = "invalid token in value";
            Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(s)))
        }
    }

    fn parse_bool(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher.eat_until(|c| cmp_tokens(c, EOL)).collect::<String>();
        let _new_line = muncher.eat();
        if s == "true" {
            Ok(Value::Bool(true))
        } else if s == "false" {
            Ok(Value::Bool(false))

        } else {
            let msg = "invalid token in value";
            Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(s)))
        }
    }
}

impl Parse for Value {
    type Item = Value;
    fn parse(muncher: &mut Muncher) -> Result<Value, ParseTomlError> {
        match muncher.peek() {
            Some('[') => {
                Ok(Value::Table(Table::parse(muncher)?))
            },
            Some(_) => {
                let msg = "toml file must start with table".into();
                Err(ParseTomlError::new(msg, TomlErrorKind::UnexpectedToken("".into())))
            },
            None => Ok(Value::Eof)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::eq_op)]
    #[test]
    fn kv_table() {
        let input =
r#"[hello]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");

        println!("{:#?}", value);
    }
}

use std::str::FromStr;

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
const DATE_LIKE: &[char] = &[ '-', '/', ' ', ':', 'T'];
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
            header.push(ch);
        }
        if header.contains('.') {
            seg = header.split('.').map(|s| s.to_string()).collect();
        }
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
    fn parse_pairs(muncher: &mut Muncher) -> TomlResult<Option<(String, Value)>> {
        if muncher.is_done() {
            return Ok(None);
        }

        let mut key = muncher.eat_until(|c| {
            println!("in key {:?}", c);
            cmp_tokens(c, KEY_END)
        }).collect::<String>();

        let val: TomlResult<Value>;
        let fork = muncher.fork();
        if fork.seek(3).map(|s| s.contains('=')) == Some(true)  {
            // eats whitespace if found
            muncher.eat_ws();
            // eats eq and optionally whitespace after.
            muncher.eat_eq();
            val = match muncher.peek() {
                Some('"') => Value::parse_str(muncher),
                Some('t') | Some('f') => Value::parse_bool(muncher),
                Some('[') => Value::parse_array(muncher),
                Some('{') => Value::parse_obj(muncher),
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
            let pair = KvPairs::parse_pairs(muncher)?;
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

        // remove last closing brace;
        assert!(muncher.eat_close_brc());
        // and new line before items
        assert!(muncher.eat_eol());

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
        if muncher.eat_quote() {
            Ok(Self::StrLit(s))
        } else {
            let msg = "invalid token in value";
            let tkn = if let Some(peek) = muncher.peek() {
                format!("{:?}", peek)
            } else {
                "no token".into()
            };
            Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(tkn)))
        }
    }

    fn parse_array(muncher: &mut Muncher) -> TomlResult<Self> {
        assert!(muncher.eat_open_brc());
        let items_raw = muncher.eat_until(|c| c == &']').collect::<String>();
        let mut items = Vec::default();
        for s in items_raw.split(',') {
            let raw = s.trim();
            let mut mini_munch = Muncher::new(raw);
            match raw.chars().next() {
                Some('"') => items.push(Value::parse_str(&mut mini_munch)?),
                Some('[') => items.push(Value::parse_array(&mut mini_munch)?),
                Some('{') => items.push(Value::parse_obj(&mut mini_munch)?),
                Some('t') | Some('f') => items.push(Value::parse_bool(&mut mini_munch)?),
                Some(digi) if digi.is_numeric() => {
                    if raw.contains(DATE_LIKE) {
                        items.push(Value::Date(NaiveDateTime::from_str(raw)?))
                    } else {
                        if raw.contains('.') {
                            items.push(Value::Float(raw.parse()?))
                        } else {
                            items.push(Value::Int(raw.parse()?))
                        }
                    }
                },
                Some(invalid) => {
                    let msg = "invalid token in value";
                    let tkn = format!("{:?}", invalid);
                    return Err(
                        ParseTomlError::new(
                            msg.into(),
                            TomlErrorKind::UnexpectedToken(tkn))
                    );
                },
                None => {
                    println!("DONE ARRAY {:?}", items);
                },
            }
        }
        assert!(muncher.eat_close_brc());
        Ok(Value::Array(items))
    }

    fn parse_bool(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher.eat_until(|c| cmp_tokens(c, EOL)).collect::<String>();
        if s == "true" {
            Ok(Value::Bool(true))
        } else if s == "false" {
            Ok(Value::Bool(false))

        } else {
            let msg = "invalid token in value";
            Err(ParseTomlError::new(msg.into(), TomlErrorKind::UnexpectedToken(s)))
        }
    }

    fn parse_obj(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher.eat_until(|c| c == &'}').collect::<String>();
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

    #[test]
    fn kv_table() {
        let input =
r#"[hello]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");

        if let Value::Table(table) = value {
            assert_eq!(table.header.header, "hello");
            assert_eq!(table.pairs.len(), 2);
        } else {
            panic!("no table parsed")
        }
    }

    #[test]
    fn seg_header() {
        let input =
r#"[hello.world]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");
        println!("{:#?}", value);
        if let Value::Table(table) = value {
            assert_eq!(table.header.header, "hello.world");
            assert_eq!(table.header.seg.len(), 2);
            assert_eq!(table.pairs.len(), 2);
        } else {
            panic!("no table parsed")
        }
    }
}

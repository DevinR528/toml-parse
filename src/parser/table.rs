use std::str::FromStr;

use chrono::{Date, DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime};

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::parse::Parse;
use super::token::{cmp_tokens, Muncher};
use super::value::Value;
use super::{KEY_END, EOL, ARRAY_ITEMS, DATE_END};

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
        Ok(Self { header, seg })
    }
}

#[derive(Debug, Clone)]
pub struct KvPairs {
    key: Option<String>,
    val: Value,
}

impl KvPairs {
    fn parse_pairs(muncher: &mut Muncher) -> TomlResult<Option<(String, Value)>> {
        if muncher.is_done() {
            return Ok(None);
        }

        if muncher.peek() == Some(&'#') {
            let cmt = muncher.eat_until(|c| cmp_tokens(c, EOL)).collect();
            assert!(muncher.eat_eol());
            return Ok(Some(("".into(), Value::Comment(cmt))));
        }

        let key = muncher
            .eat_until(|c| {
                println!("in key {:?}", c);
                cmp_tokens(c, KEY_END)
            })
            .collect::<String>();

        let val: TomlResult<Value>;
        let fork = muncher.fork();
        if fork.seek(3).map(|s| s.contains('=')) == Some(true) {
            // eats whitespace if found
            muncher.eat_ws();
            // eats eq and optionally whitespace after.
            muncher.eat_eq();
            muncher.reset_peek();
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
                    Err(ParseTomlError::new(
                        msg.into(),
                        TomlErrorKind::UnexpectedToken(tkn),
                    ))
                }
            }
        } else if fork.peek().map(|c| cmp_tokens(c, EOL)) == Some(true) {
            return Ok(None);
        } else {
            let msg = "invalid token in key value pairs";
            let tkn = if let Some(peek) = muncher.peek() {
                format!("{:?}", peek)
            } else {
                "no token".into()
            };
            val = Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken(tkn),
            ));
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
            if muncher.peek() == Some(&'\n') {
                break;
            }
            let pair = KvPairs::parse_pairs(muncher)?;
            if let Some((key, val)) = pair {
                let key = if key.is_empty() { None } else { Some(key) };
                pairs.push(Self { key, val });
                // remove new line after each pair
                muncher.eat_eol();
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

impl Table {
    pub(crate) fn header(&self) -> &str {
        &self.header.header
    }
    pub(crate) fn segments(&self) -> &[String] {
        &self.header.seg
    }
    pub(crate) fn item_len(&self) -> usize {
        self.pairs.len()
    }
    pub(crate) fn seg_len(&self) -> usize {
        self.header.seg.len()
    }
    pub(crate) fn items(&self) -> &[KvPairs] {
        &self.pairs
    }
}

impl Parse for Table {
    type Item = Table;
    fn parse(muncher: &mut Muncher) -> Result<Table, ParseTomlError> {
        assert!(muncher.eat_open_brc());
        let header = Heading::parse(muncher)?;
        // remove last closing brace;
        assert!(muncher.eat_close_brc());
        // and new line before items
        assert!(muncher.eat_eol());
        let pairs = KvPairs::parse(muncher)?;
        // TODO this may not always be needed
        assert!(muncher.eat_eol());

        Ok(Self { header, pairs })
    }
}

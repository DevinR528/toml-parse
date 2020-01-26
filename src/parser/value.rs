use std::str::FromStr;

use chrono::{Date, DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime};

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::parse::Parse;
use super::token::{cmp_tokens, Muncher};
use super::table::{Table, KvPairs};
use super::{EOL, NUM_END, DATE_END, DATE_LIKE, ARRAY_ITEMS, OBJ_ITEMS};
use super::date::TomlDate;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    StrLit(String),
    Date(TomlDate),
    Array(Vec<Value>),
    Object(Vec<KvPairs>),
    Table(Table),
    Comment(String),
    Keys(Vec<KvPairs>),
    Eof,
}

impl Value {
    pub (crate) fn parse_bool(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher
            .eat_until(|c| cmp_tokens(c, EOL))
            .collect::<String>();
        if s == "true" {
            Ok(Value::Bool(true))
        } else if s == "false" {
            Ok(Value::Bool(false))
        } else {
            let msg = "invalid token in value";
            Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken(s),
            ))
        }
    }

    pub (crate) fn parse_int(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher.eat_until(|c| cmp_tokens(c, NUM_END)).collect::<String>();
        let s = s.replace('_', "");
        let cleaned = s.trim_left_matches('+');

        if s.starts_with("0x") {
            let without_prefix = cleaned.chars().skip(2).collect::<String>();
            let z = i64::from_str_radix(&without_prefix, 16);
            return Ok(Value::Int(z?))
        } else if s.starts_with("0o") {
            let without_prefix = cleaned.chars().skip(2).collect::<String>();
            let z = i64::from_str_radix(&without_prefix, 8);
            return Ok(Value::Int(z?))
        } else if s.starts_with("0b") {
            let without_prefix = cleaned.chars().skip(2).collect::<String>();
            let z = i64::from_str_radix(&without_prefix, 2);
            return Ok(Value::Int(z?))
        }
        
        Ok(Value::Int(cleaned.parse()?))
    }

    pub (crate) fn parse_float(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher.eat_until(|c| cmp_tokens(c, NUM_END)).collect::<String>();
        let cleaned = s.replace('_', "");
        Ok(Value::Float(cleaned.parse()?))
    }

    pub (crate) fn parse_date(muncher: &mut Muncher) -> TomlResult<Self> {
        let mut s = muncher.eat_until(|c| cmp_tokens(c, DATE_END)).collect::<String>();
        if s.ends_with(' ') {
            let (keep, junk) = s.split_at(s.chars().count() - 1);
            println!("{:?}   {:?}", keep, junk);
            s = keep.to_string();
        }
        Ok(Value::Date(TomlDate::from_str(&s)?))
    }

    pub (crate) fn parse_str(muncher: &mut Muncher) -> TomlResult<Self> {
        let mut pair = 0;
        let mut s = muncher
            .eat_until(|c| {
                if c == &'"' {
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
            Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken(tkn),
            ))
        }
    }

    /// TODO use muncher like every other parse function
    pub (crate) fn parse_array(muncher: &mut Muncher) -> TomlResult<Self> {
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
                        items.push(Value::parse_date(&mut mini_munch)?)
                    } else if raw.contains('.') {
                        items.push(Value::parse_float(&mut mini_munch)?)
                    } else {
                        items.push(Value::parse_int(&mut mini_munch)?)
                    }
                }
                Some(invalid) => {
                    let msg = "invalid token in value";
                    let tkn = format!("{:?}", invalid);
                    return Err(ParseTomlError::new(
                        msg.into(),
                        TomlErrorKind::UnexpectedToken(tkn),
                    ));
                }
                None => {
                    println!("DONE ARRAY {:?}", items);
                }
            }
        }
        assert!(muncher.eat_close_brc());
        Ok(Value::Array(items))
    }

    pub (crate) fn parse_obj(muncher: &mut Muncher) -> TomlResult<Self> {
        let pair = muncher.eat_until(|c| cmp_tokens(c, OBJ_ITEMS));
        let kv = KvPairs::parse(muncher)?;

        assert!(muncher.eat_close_curly());

        Ok(Value::Object(kv))
    }
}

impl Parse for Value {
    type Item = Value;
    fn parse(muncher: &mut Muncher) -> Result<Value, ParseTomlError> {
        match muncher.peek() {
            Some('#') => {
                let cmt = Ok(Value::Comment(
                    muncher.eat_until(|c| cmp_tokens(c, EOL))
                        .collect::<String>()
                    ));
                assert!(muncher.eat_eol());
                cmt
            },
            Some('[') => Ok(Value::Table(Table::parse(muncher)?)),
            Some(ch) if ch.is_ascii() => Ok(Value::Keys(KvPairs::parse(muncher)?)),
            Some(_) => {
                let msg = "toml file must start with table".into();
                Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken("".into()),
                ))
            }
            None => Ok(Value::Eof),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_bool() {
        let input = "true";
        let mut muncher = Muncher::new(input);
        let value = Value::parse_bool(&mut muncher);
        println!("{:#?}", value);
    }

    #[test]
    fn value_array() {
        let input = r#"[ "a", "b", "c", ]"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_array(&mut muncher);
        println!("{:#?}", value);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn value_float() {
        let input = r#"224_617.445_991_228"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_float(&mut muncher).expect("float parse");
        if let Value::Float(flt) = value {
            assert_eq!(flt, 224_617.445_991_228)
        }
    }

    #[test]
    fn value_integer() {
        let ints = &[
            "5_349_221",
            "0xdeadbeef",
            "0b11010110"
        ];
        let integers = &[
            5_349_221,
            0xdead_beef,
            0b1101_0110
        ];
        for (i, int) in ints.iter().enumerate() {
            let mut muncher = Muncher::new(int);
            let value = Value::parse_int(&mut muncher).expect("int parse failed");
            if let Value::Int(num) = value {
                assert_eq!(num, integers[i])
            }
        }
    }

    #[test]
    fn value_date() {
        let dates = &[
            "1979-05-27T07:32:01+09:30,", // with offset date-time
            "1979-05-27T07:32:01,",  // local date-time
            "1979-05-27,",           // local date
            "00:32:00.999\n",      // local time
        ];
        let fmt = &[
            "%Y-%m-%dT%H:%M:%S%z", // with offset date-time
            "%Y-%m-%dT%H:%M:%S",  // local date-time
            "%Y-%m-%d",           // local date
            "%H:%M:%S%.f",      // local time
        ];
        
        for (input, fmt) in dates.iter().zip(fmt.iter()) {
            println!("{:?} {:?}", input, fmt);
            let mut fixed = (*input).to_string();
            fixed.pop();
            let mut muncher = Muncher::new(input);
            let value = Value::parse_date(&mut muncher).expect("Parse Failed");
            println!("{:#?}", value);
            if let Value::Date(dt) = value {
                match dt {
                    TomlDate::DateTime(dt) => {
                        let parsed_dt = NaiveDateTime::parse_from_str(&fixed, fmt).expect("naive dt failed");
                        assert_eq!(dt, parsed_dt)
                    },
                    TomlDate::Time(dt) => {
                        let parsed_dt = NaiveTime::parse_from_str(&fixed, fmt).expect("naive dt failed");
                        assert_eq!(dt, parsed_dt)
                    },
                    TomlDate::Date(dt) => {
                        let parsed_dt = NaiveDate::parse_from_str(&fixed, fmt).expect("naive dt failed");
                        assert_eq!(dt, parsed_dt)
                    },
                }
            } else {
                panic!("date not parsed")
            }
        }
    }
}

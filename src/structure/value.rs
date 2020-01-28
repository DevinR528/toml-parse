use super::date::TomlDate;
use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::munch::{cmp_tokens, Muncher};
use super::parse::Parse;
use super::table::{InTable, KvPairs, Table};
use super::{ARRAY_ITEMS, BOOL_END, DATE_LIKE, EOL, NUM_END};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    StrLit(String),
    Date(TomlDate),
    Array(Vec<Value>),
    InlineTable(InTable),
    Table(Table),
    Comment(String),
    KeyValue(Box<KvPairs>),
    Eof,
}

impl Value {
    pub(crate) fn parse_bool(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher
            .eat_until(|c| cmp_tokens(c, BOOL_END))
            .collect::<String>();
        if s == "true" {
            Ok(Value::Bool(true))
        } else if s == "false" {
            Ok(Value::Bool(false))
        } else {
            let msg = "invalid token in value";
            let (ln, col) = muncher.cursor_position();
            Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken { tkn: s, ln, col },
            ))
        }
    }

    pub(crate) fn parse_int(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher
            .eat_until(|c| cmp_tokens(c, NUM_END))
            .collect::<String>();
        let s = s.replace('_', "");
        let cleaned = s.trim_start_matches('+');

        if s.starts_with("0x") {
            let without_prefix = cleaned.chars().skip(2).collect::<String>();
            let z = i64::from_str_radix(&without_prefix, 16);
            return Ok(Value::Int(z?));
        } else if s.starts_with("0o") {
            let without_prefix = cleaned.chars().skip(2).collect::<String>();
            let z = i64::from_str_radix(&without_prefix, 8);
            return Ok(Value::Int(z?));
        } else if s.starts_with("0b") {
            let without_prefix = cleaned.chars().skip(2).collect::<String>();
            let z = i64::from_str_radix(&without_prefix, 2);
            return Ok(Value::Int(z?));
        }

        Ok(Value::Int(cleaned.parse()?))
    }

    pub(crate) fn parse_float(muncher: &mut Muncher) -> TomlResult<Self> {
        let s = muncher
            .eat_until(|c| cmp_tokens(c, NUM_END))
            .collect::<String>();
        let cleaned = s.replace('_', "");
        Ok(Value::Float(cleaned.parse()?))
    }

    pub(crate) fn parse_date(muncher: &mut Muncher) -> TomlResult<Self> {
        let mut s = muncher
            .eat_until(|c| cmp_tokens(c, NUM_END))
            .collect::<String>();
        if s.ends_with(' ') {
            println!("COLLECTED SPACE IN NUMBER {} BUG?", s);
            s = s.replace(" ", "");
        }
        Ok(Value::Date(TomlDate::from_str(&s)?))
    }

    pub(crate) fn parse_str(muncher: &mut Muncher) -> TomlResult<Self> {
        muncher.reset_peek();
        let triple_quote = muncher.seek(3).map(|s| s == "\"\"\"");
        let mut s = if triple_quote == Some(true) {
            for _ in 0..=3 {
                muncher.eat_quote();
            }
            let mut trip = 0;
            muncher
                .eat_until(|c| {
                    if c == &'"' {
                        trip += 1;
                        trip == 3
                    } else {
                        trip = 0;
                        false
                    }
                })
                .collect::<String>()
        } else {
            let mut pair = 0;
            muncher
                .eat_until(|c| {
                    if c == &'"' {
                        pair += 1;
                        pair == 2
                    } else {
                        false
                    }
                })
                .collect::<String>()
        };

        if s.starts_with('"') {
            s.remove(0);
        }
        if muncher.eat_quote() {
            if triple_quote == Some(true) {
                s.pop();
                s.pop();
            }
            Ok(Self::StrLit(s))
        } else {
            let msg = "invalid token in value";
            let tkn = if let Some(peek) = muncher.peek() {
                format!("{:?}", peek)
            } else {
                "no token".into()
            };
            let (ln, col) = muncher.cursor_position();
            Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken { tkn, ln, col },
            ))
        }
    }

    /// TODO use muncher like every other parse function
    pub(crate) fn parse_array(muncher: &mut Muncher) -> TomlResult<Self> {
        assert!(muncher.eat_open_brc());
        if !muncher.eat_ws() {
            muncher.reset_peek();
        }
        // let items_raw = muncher.eat_until(|c| c == &']').collect::<String>();
        let mut items = Vec::default();
        loop {
            // let item = muncher.eat_until(|c| cmp_tokens(c, ARRAY_ITEMS)).collect::<String>();
            match muncher.peek() {
                Some(']') => break,
                Some('"') => items.push(Value::parse_str(muncher)?),
                Some('[') => items.push(Value::parse_array(muncher)?),
                Some('{') => items.push(Value::InlineTable(InTable::parse(muncher)?)),
                Some('t') | Some('f') => items.push(Value::parse_bool(muncher)?),
                Some(digi) if digi.is_numeric() => {
                    let raw = muncher
                        .peek_until(|c| cmp_tokens(c, ARRAY_ITEMS))
                        .collect::<String>();
                    if raw.contains(DATE_LIKE) {
                        items.push(Value::parse_date(muncher)?)
                    } else if raw.contains('.') {
                        items.push(Value::parse_float(muncher)?)
                    } else {
                        items.push(Value::parse_int(muncher)?)
                    }
                }
                Some(',') => {
                    muncher.eat_comma();
                }
                Some(' ') => {
                    muncher.eat_ws();
                }
                Some(invalid) => {
                    let msg = "invalid token in value";
                    let tkn = format!("{:?}", invalid);
                    let (ln, col) = muncher.cursor_position();
                    return Err(ParseTomlError::new(
                        msg.into(),
                        TomlErrorKind::UnexpectedToken { tkn, ln, col },
                    ));
                }
                None => break,
            }
        }

        assert!(muncher.eat_close_brc());
        Ok(Value::Array(items))
    }
}

impl Parse for Value {
    type Item = Value;
    fn parse(muncher: &mut Muncher) -> Result<Value, ParseTomlError> {
        match muncher.peek() {
            Some('#') => {
                let cmt = Ok(Value::Comment(
                    muncher
                        .eat_until(|c| cmp_tokens(c, EOL))
                        .collect::<String>(),
                ));
                assert!(muncher.eat_eol());
                cmt
            }
            Some('[') => Ok(Value::Table(Table::parse(muncher)?)),
            Some(ch) if ch.is_ascii() => {
                let kv = KvPairs::parse_one(muncher)?;
                Ok(Value::KeyValue(Box::new(kv)))
            }
            Some(tkn) => {
                let msg = "toml file must be key values or tables".into();
                let tkn = format!("{}", tkn);
                let (ln, col) = muncher.cursor_position();
                Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
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
        let value = Value::parse_bool(&mut muncher).expect("bool failed");
        if let Value::Bool(b) = value {
            assert_eq!(b, true)
        }
    }

    #[test]
    fn value_triple_quote() {
        let input = r#""""hello""""#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_str(&mut muncher).expect("triple quote failed");
        // println!("{:#?}", value);
        if let Value::StrLit(s) = value {
            assert_eq!(s, "hello")
        }
    }

    #[test]
    fn value_bool_fail() {
        let input = "untrue";
        let mut muncher = Muncher::new(input);
        let value = Value::parse_bool(&mut muncher);
        // println!("{:#?}", value);
        assert!(value.is_err());
    }

    #[test]
    fn value_array() {
        let input = r#"[ "a", "b", "c", ]"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_array(&mut muncher).expect("array failed");
        // println!("{:#?}", value);
        if let Value::Array(arr) = value {
            assert_eq!(arr.len(), 3);
            // println!("{:?}", arr)
        }
    }

    #[test]
    fn nested_array() {
        let input = r#"[ ["a"], ["b", "c"] ]"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_array(&mut muncher).expect("array failed");
        // println!("{:#?}", value);
        if let Value::Array(arr) = value {
            assert_eq!(arr.len(), 2);
            // println!("{:?}", arr)
        }
    }

    #[test]
    fn value_array_fail() {
        let input = r#"[ will_fail ]"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_array(&mut muncher);
        // println!("{:#?}", value);
        assert!(value.is_err())
    }

    #[test]
    fn value_object() {
        let input = r#"{ a = "a", b = "b" }"#;
        let mut muncher = Muncher::new(input);
        let value = InTable::parse(&mut muncher).expect("obj failed");
        // println!("{:#?}", value);
        assert_eq!(value.len(), 2);

        let mut m = Muncher::new(r#"a = "a""#);
        let a_pair = KvPairs::parse(&mut m).expect("cmp kv failed");
        assert_eq!(value.get("a"), Some(&a_pair[0]))
    }

    #[test]
    fn nested_object() {
        let input = r#"{ a = { c = "c" }, b = "b" }"#;
        let mut muncher = Muncher::new(input);
        let value = InTable::parse(&mut muncher).expect("obj failed");
        // println!("{:#?}", value);
        assert_eq!(value.len(), 2);

        let mut m = Muncher::new(r#"b = "b""#);
        let a_pair = KvPairs::parse(&mut m).expect("cmp kv failed");
        assert_eq!(value.get("b"), Some(&a_pair[0]));
        assert_eq!(value.get("a").unwrap().key(), Some("a"));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn value_float() {
        let input = r#"224_617.445_991_228"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse_float(&mut muncher).expect("float failed");
        if let Value::Float(flt) = value {
            assert_eq!(flt, 224_617.445_991_228)
        }
    }

    #[test]
    fn value_integer() {
        let ints = &["5_349_221", "0xdeadbeef", "0b11010110"];
        let integers = &[5_349_221, 0xdead_beef, 0b1101_0110];
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
        use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

        let dates = &[
            "1979-05-27T07:32:01+09:30,", // with offset date-time
            "1979-05-27T07:32:01,",       // local date-time
            "1979-05-27,",                // local date
            "00:32:00.999\n",             // local time
        ];
        let fmt = &[
            "%Y-%m-%dT%H:%M:%S%z", // with offset date-time
            "%Y-%m-%dT%H:%M:%S",   // local date-time
            "%Y-%m-%d",            // local date
            "%H:%M:%S%.f",         // local time
        ];

        for (input, fmt) in dates.iter().zip(fmt.iter()) {
            let mut fixed = (*input).to_string();
            fixed.pop();
            let mut muncher = Muncher::new(input);
            let value = Value::parse_date(&mut muncher).expect("Parse Failed");
            if let Value::Date(dt) = value {
                match dt {
                    TomlDate::DateTime(dt) => {
                        let parsed_dt =
                            NaiveDateTime::parse_from_str(&fixed, fmt).expect("naive dt failed");
                        assert_eq!(dt, parsed_dt)
                    }
                    TomlDate::Time(dt) => {
                        let parsed_dt =
                            NaiveTime::parse_from_str(&fixed, fmt).expect("naive dt failed");
                        assert_eq!(dt, parsed_dt)
                    }
                    TomlDate::Date(dt) => {
                        let parsed_dt =
                            NaiveDate::parse_from_str(&fixed, fmt).expect("naive dt failed");
                        assert_eq!(dt, parsed_dt)
                    }
                }
            } else {
                panic!("date not parsed")
            }
        }
    }

    #[test]
    fn kv_table() {
        let input = r#"[hello]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");

        if let Value::Table(table) = value {
            assert_eq!(table.header(), "hello");
            assert_eq!(table.item_len(), 2);
        } else {
            panic!("no table parsed")
        }
    }

    #[test]
    fn seg_header() {
        let input = r#"[hello.world]
a = "a"
b = "b"
"#;
        let mut muncher = Muncher::new(input);
        let value = Value::parse(&mut muncher).expect("Parse Failed");
        if let Value::Table(table) = value {
            assert_eq!(table.header(), "hello.world");
            assert_eq!(table.seg_len(), 2);
            assert_eq!(table.item_len(), 2);
        } else {
            panic!("no table parsed")
        }
    }

    #[test]
    fn file_parser() {
        // ftop.toml is 7 items long
        let input = std::fs::read_to_string("examp/ftop.toml").expect("file read failed");

        let mut parsed = Vec::default();
        let mut muncher = Muncher::new(&input);
        while let Ok(value) = Value::parse(&mut muncher) {
            if value == Value::Eof {
                break;
            };
            parsed.push(value);
        }
        assert_eq!(parsed.len(), 7);
        for item in parsed {
            // println!("{:#?}", item);
        }
    }
}

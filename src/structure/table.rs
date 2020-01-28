use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::munch::{cmp_tokens, Muncher};
use super::parse::Parse;
use super::value::Value;
use super::{DATE_LIKE, EOL, KEY_END, NUM_END};

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct KvPairs {
    key: Option<String>,
    val: Value,
}

impl KvPairs {
    fn key_match(&self, key: &str) -> bool {
        self.key.as_ref().map(|k| k == key) == Some(true)
    }

    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }
    pub fn value(&self) -> &Value {
        &self.val
    }
}

impl KvPairs {
    fn parse_pairs(muncher: &mut Muncher) -> TomlResult<Option<(String, Value)>> {
        if muncher.is_done() {
            return Ok(None);
        }

        let peeked = muncher.peek();
        if peeked == Some(&'#') {
            let cmt = muncher.eat_until(|c| cmp_tokens(c, EOL)).collect();
            assert!(muncher.eat_eol());
            return Ok(Some(("".into(), Value::Comment(cmt))));
        }

        let key = muncher
            .eat_until(|c| cmp_tokens(c, KEY_END))
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
                Some('{') => Ok(Value::InlineTable(InTable::parse(muncher)?)),
                Some(digi) if digi.is_numeric() => {
                    muncher.reset_peek();
                    let raw = muncher
                        .peek_until(|c| cmp_tokens(c, NUM_END))
                        .collect::<String>();
                    if raw.contains(DATE_LIKE) {
                        Value::parse_date(muncher)
                    } else if raw.contains('.') {
                        Value::parse_float(muncher)
                    } else {
                        Value::parse_int(muncher)
                    }
                }
                None => Ok(Value::Eof),
                _ => {
                    let msg = "invalid token in key value pairs";
                    let tkn = if let Some(peek) = muncher.peek() {
                        format!("{:#?}", peek)
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
        } else if fork.peek().map(|c| cmp_tokens(c, EOL)) == Some(true) {
            // TODO is this \n\n this is the EOF for the file??
            return Ok(None);
        } else {
            let msg = "invalid token in key value pairs";
            let tkn = if let Some(peek) = muncher.peek() {
                format!("{:?}", peek)
            } else {
                "no token".into()
            };
            let (ln, col) = muncher.cursor_position();
            val = Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken { tkn, ln, col },
            ));
        }

        if let Ok(Value::Eof) = val {
            return Ok(None);
        }
        // println!("{:?} {:?}", key, val);
        Ok(Some((key, val?)))
    }

    pub(crate) fn parse_one(muncher: &mut Muncher) -> TomlResult<KvPairs> {
        muncher.reset_peek();
        let pair = KvPairs::parse_pairs(muncher)?;
        if let Some((key, val)) = pair {
            let key = if key.is_empty() { None } else { Some(key) };
            let kv = Self { key, val };
            // remove new line after each pair
            muncher.eat_eol();
            Ok(kv)
        } else {
            todo!("what happens here");
        }
    }
}

impl Parse for KvPairs {
    type Item = Vec<KvPairs>;
    fn parse(muncher: &mut Muncher) -> TomlResult<Vec<KvPairs>> {
        let mut pairs = Vec::default();
        loop {
            if muncher.peek() == Some(&'\n') {
                break;
            }
            muncher.reset_peek();
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

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    header: Heading,
    pairs: Vec<KvPairs>,
}

impl Table {
    /// The heading of the given `Table`.
    pub fn header(&self) -> &str {
        &self.header.header
    }
    /// The segments of the heading of a given `Table`.
    pub fn segments(&self) -> &[String] {
        &self.header.seg
    }
    /// The number of items in this `Table`.
    pub fn item_len(&self) -> usize {
        self.pairs.len()
    }
    /// Number of segments the header is broken into.
    ///
    /// ```ignore
    /// [this.is.segmented]
    /// key = "value"
    /// ```
    pub fn seg_len(&self) -> usize {
        self.header.seg.len()
    }
    /// The `KvPairs` this table holds as a slice.
    pub fn items(&self) -> &[KvPairs] {
        &self.pairs
    }

    pub fn get(&self, key: &str) -> Option<&KvPairs> {
        self.pairs.iter().find(|pair| pair.key_match(key))
    }
}

impl Parse for Table {
    type Item = Table;
    fn parse(muncher: &mut Muncher) -> TomlResult<Table> {
        assert!(muncher.eat_open_brc());
        let header = Heading::parse(muncher)?;
        // remove last closing brace;
        assert!(muncher.eat_close_brc());
        // and new line before items
        assert!(muncher.eat_eol());
        let pairs = KvPairs::parse(muncher)?;
        // TODO this may not always be needed
        muncher.eat_eol();

        Ok(Self { header, pairs })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InTable {
    pairs: Vec<KvPairs>,
}

impl InTable {
    pub fn len(&self) -> usize {
        self.pairs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, key: &str) -> Option<&KvPairs> {
        self.pairs.iter().find(|pair| pair.key_match(key))
    }
}

impl Parse for InTable {
    type Item = InTable;
    fn parse(muncher: &mut Muncher) -> TomlResult<InTable> {
        assert!(muncher.eat_open_curly());
        muncher.eat_ws();

        let mut pairs = Vec::default();
        loop {
            if muncher.peek() == Some(&'}') {
                break;
            }
            muncher.reset_peek();

            let pair = KvPairs::parse_pairs(muncher)?;

            if let Some((key, val)) = pair {
                let key = if key.is_empty() { None } else { Some(key) };
                pairs.push(KvPairs { key, val });
                // remove optional comma
                muncher.eat_comma();
                // remove new line after each pair
                muncher.eat_ws();
            } else {
                break;
            }
        }
        // remove optional comma
        muncher.eat_comma();
        // remove new line after each pair
        muncher.eat_ws();
        assert!(muncher.eat_close_curly());

        Ok(InTable { pairs })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn num_kvpair() {
        let input = "key = 12345";

        let mut muncher = Muncher::new(input);
        let kv = KvPairs::parse(&mut muncher);
        println!("{:?}", kv);
    }
}

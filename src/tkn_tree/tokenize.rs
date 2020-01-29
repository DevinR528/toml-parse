use smol_str::SmolStr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::kinds::{Element, TomlKind::*, TomlNode, TomlToken};
use super::munch::{
    cmp_tokens, Muncher, ARRAY_ITEMS, BOOL_END, EOL, INT_END, NUM_END, QUOTE, WHITESPACE, DATE_LIKE,
    DATE_TIME, DATE_END, DATE_CHAR, TIME_CHAR, KEY_END, INLINE_ITEMS
};

fn is_valid_key(s: &str) -> bool {
    s.chars().all(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' |'_' | '.' | '-' => true,
        _ => false,
    })
}

fn is_valid_datetime(s: &str) -> TomlResult<bool> {
    let dt = s.split(DATE_TIME).collect::<Vec<_>>();
    if dt.len() == 1 {
        if dt[0].contains(':') {
            let time = dt[0].split(":").collect::<Vec<_>>();
            if time[2].contains('.') {
                let (sec, milli) = {
                    let fractional = time[2].split('.').collect::<Vec<_>>();
                    (fractional[0].parse()?, fractional[1].parse()?)
                };
                println!("has fractional seconds {:?} {:?}", sec, milli);
                NaiveTime::from_hms_milli(time[0].parse()?, time[1].parse()?, sec, milli);
            } else {
                NaiveTime::from_hms(time[0].parse()?, time[1].parse()?, time[2].parse()?);
            };
            Ok(true)
        } else {
            let date = dt[0].split("-").collect::<Vec<_>>();

            assert_eq!(date.len(), 3);

            let _ = NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?);
            Ok(true)
        }
    } else {
        let date = dt[0].split(DATE_CHAR).collect::<Vec<_>>();
        let time = dt[1].split(TIME_CHAR).collect::<Vec<_>>();
        let _ =
            if time.len() > 3 {
                if s.contains('+') {
                    // TODO dont include offset for now
                    NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?)
                        .and_hms(time[0].parse()?, time[1].parse()?, time[2].parse()?)
                } else {
                    NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?)
                        .and_hms_milli(
                            time[0].parse()?,
                            time[1].parse()?,
                            time[2].parse()?,
                            time[3].parse()?,
                        )
                }
            } else {
                NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?)
                    .and_hms(time[0].parse()?, time[1].parse()?, time[2].parse()?)
            };
        Ok(true)
    }
}

impl TomlToken {
    fn whitespace(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| !cmp_tokens(c, WHITESPACE));
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("WS {:?}", text);
        Ok(Element::Token(Self {
            kind: Whitespace,
            text,
            range: s..e,
        }))
    }

    /// Returns Element if whitespace was found.
    fn maybe_whitespace(muncher: &mut Muncher) -> Option<Element> {
        let (s, e) = muncher.eat_until_count(|c| !cmp_tokens(c, WHITESPACE));
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("WS {:?}", text);

        if s != e {
            Some(Element::Token(Self {
                kind: Whitespace,
                text,
                range: s..e,
            }))
        } else {
            None
        }
    }

    fn hash(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_hash());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: Hash,
            text: SmolStr::new("#"),
            range: start..end,
        }))
    }

    fn plus(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_plus());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: Plus,
            text: SmolStr::new("+"),
            range: start..end,
        }))
    }

    fn minus(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_minus());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: Minus,
            text: SmolStr::new("-"),
            range: start..end,
        }))
    }

    fn equal(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_eq());
        let end = start + 1;
        println!("EQ");
        Ok(Element::Token(Self {
            kind: Equal,
            text: SmolStr::new("="),
            range: start..end,
        }))
    }

    fn comma(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_comma());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: Comma,
            text: SmolStr::new(","),
            range: start..end,
        }))
    }

    /// Returns Element if comma was found. The last item
    /// in an array may or may not have a comma.
    fn maybe_comma(muncher: &mut Muncher) -> Option<Element> {
        let start = muncher.position();
        if muncher.eat_comma() {
            let end = start + 1;
            Some(Element::Token(Self {
                kind: Comma,
                text: SmolStr::new(","),
                range: start..end,
            }))
        } else {
            None
        }
    }

    fn colon(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_colon());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: Colon,
            text: SmolStr::new(":"),
            range: start..end,
        }))
    }

    fn dot(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_dot());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: Dot,
            text: SmolStr::new("."),
            range: start..end,
        }))
    }

    fn maybe_dot(muncher: &mut Muncher) -> Option<Element> {
        let start = muncher.position();
        if muncher.eat_dot() {
            let end = start + 1;
            Some(Element::Token(Self {
                kind: Dot,
                text: SmolStr::new("."),
                range: start..end,
            }))
        } else {
            None
        }
        
    }

    fn double_quote(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_quote());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: DoubleQuote,
            text: SmolStr::new("\""),
            range: start..end,
        }))
    }

    fn single_quote(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_quote());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: SingleQuote,
            text: SmolStr::new("\'"),
            range: start..end,
        }))
    }

    fn ident(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, WHITESPACE));
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        Ok(Element::Token(Self {
            kind: Ident,
            text,
            range: s..e,
        }))
    }

    fn ident_str(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| c == &'"');
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        Ok(Element::Token(Self {
            kind: Ident,
            text,
            range: s..e,
        }))
    }

    fn ident_heading(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| c == &']');
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        Ok(Element::Token(Self {
            kind: Ident,
            text,
            range: s..e,
        }))
    }

    fn comment_text(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, EOL));
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);

        Ok(Element::Token(Self {
            kind: CommentText,
            text,
            range: s..e,
        }))
    }

    fn open_brace(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        println!("{:?}", start);
        assert!(muncher.eat_open_brc());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: OpenBrace,
            text: SmolStr::new("["),
            range: start..end,
        }))
    }
    fn close_brace(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_close_brc());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: CloseBrace,
            text: SmolStr::new("]"),
            range: start..end,
        }))
    }
    fn open_curly(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_open_curly());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: OpenCurly,
            text: SmolStr::new("{"),
            range: start..end,
        }))
    }
    fn close_curly(muncher: &mut Muncher) -> TomlResult<Element> {
        let start = muncher.position();
        assert!(muncher.eat_close_curly());
        let end = start + 1;
        Ok(Element::Token(Self {
            kind: CloseCurly,
            text: SmolStr::new("}"),
            range: start..end,
        }))
    }
    fn boolean(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, BOOL_END));
        let boolean = &muncher.text()[s..e];

        let text = SmolStr::new(boolean);
        println!("BOOL {:?}", text);
        if boolean == "true" || boolean == "false" {
            Ok(Element::Token(Self {
                kind: Bool,
                text,
                range: s..e,
            }))
        } else {
            let (ln, col) = muncher.cursor_position();
            let msg = "invalid integer".into();
            Err(ParseTomlError::new(
                msg,
                TomlErrorKind::UnexpectedToken {
                    tkn: boolean.into(),
                    ln,
                    col,
                },
            ))
        }
    }
    fn integer(muncher: &mut Muncher) -> TomlResult<Element> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, INT_END));
        let int = &muncher.text()[s..e];

        if int.chars().all(|c| c.is_numeric()) {
            let text = SmolStr::new(int);
            Ok(Element::Token(Self {
                kind: Integer,
                text,
                range: s..e,
            }))
        } else {
            let (ln, col) = muncher.cursor_position();
            let msg = "invalid integer".into();
            Err(ParseTomlError::new(
                msg,
                TomlErrorKind::UnexpectedToken {
                    tkn: int.into(),
                    ln,
                    col,
                },
            ))
        }
    }
}

/// All `TomlNodes` parse `Whitespace` token from the previous tokens
/// whitespace.
impl TomlNode {
    /// Builds `Whitespace` and `Hash` token and adds them as
    /// children.
    fn comment(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, EOL));
        children.push(TomlToken::hash(muncher)?);
        children.push(TomlToken::comment_text(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("COMMENT {:?}", text);
        Ok(Element::Node(Self {
            kind: Comment,
            text,
            range: s..e,
            children,
        }))
    }

    fn float(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, NUM_END));
        children.push(TomlToken::integer(muncher)?);
        children.push(TomlToken::dot(muncher)?);
        children.push(TomlToken::integer(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("FLOAT {:?}", text);
        Ok(Element::Node(Self {
            kind: Float,
            text,
            range: s..e,
            children,
        }))
    }

    fn date_time(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, DATE_END));

        let text = SmolStr::new(&muncher.text()[s..e]);
        
        if is_valid_datetime(&text) != Ok(true) {
            let (ln, col) = muncher.cursor_position();
            let msg = "invalid integer".into();
            return Err(ParseTomlError::new(
                msg,
                TomlErrorKind::UnexpectedToken {
                    tkn: text.into(),
                    ln,
                    col,
                },
            ));
        }
        
        println!("DATETIME {:?}", text);
        Ok(Element::Node(Self {
            kind: Date,
            text,
            range: s..e,
            children,
        }))
    }

    fn double_str(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        let mut quote = 0;
        let (s, e) = muncher.peek_until_count(|c| {
            if c == &'"' {
                quote += 1;
                quote == 2
            } else {
                false
            }
        });
        println!("{:?}", &muncher.text()[s..e]);
        children.push(TomlToken::double_quote(muncher)?);
        children.push(TomlToken::ident_str(muncher)?);
        children.push(TomlToken::double_quote(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        // println!("STR {:?}", text);
        Ok(Element::Node(Self {
            kind: Str,
            text,
            range: s..e,
            children,
        }))
    }

    fn key(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, KEY_END));
        match muncher.peek() {
            Some(&'"') => children.push(TomlNode::double_str(muncher)?),
            Some(ch) if ch.is_ascii() => children.push(TomlToken::ident(muncher)?),
            Some(&'\'') => {
                let (ln, col) = muncher.cursor_position();
                let msg = "invalid key token".into();
                return Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken {
                        tkn: "\'".into(),
                        ln,
                        col,
                    },
                ));
            }
            Some(tkn) => {
                let (ln, col) = muncher.cursor_position();
                let msg = "invalid key token".into();
                let tkn = format!("{}", tkn);
                return Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
            None => println!("what if no peek in key"),
        }

        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("KEY {:?}", text);
        if is_valid_key(&text) {
            Ok(Element::Node(Self {
                kind: Key,
                text,
                range: s..e,
                children,
            }))
        } else {
            let (ln, col) = muncher.cursor_position();
                let msg = "invalid key token".into();
                let tkn = format!("{}", text);
                Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ))
        }
        
    }

    fn value(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, EOL));
        muncher.reset_peek();
        match muncher.peek() {
            Some('"') => children.push(TomlNode::double_str(muncher)?),
            Some('t') | Some('f') => children.push(TomlToken::boolean(muncher)?),
            Some('[') => children.push(TomlNode::array(muncher)?),
            Some('{') => children.push(TomlNode::inline_table(muncher)?),
            Some(digi) if digi.is_numeric() => {
                muncher.reset_peek();
                let raw = muncher
                    .peek_until(|c| cmp_tokens(c, NUM_END))
                    .collect::<String>();
                if raw.contains(DATE_LIKE) {
                    children.push(TomlNode::date_time(muncher)?)
                } else if raw.contains('.') {
                    children.push(TomlNode::float(muncher)?)
                } else {
                    children.push(TomlToken::integer(muncher)?)
                }
            }
            None => children.push(Element::Token(TomlToken {
                kind: EoF,
                text: SmolStr::default(),
                range: 0..0,
            })),
            _ => {
                let msg = "invalid token in key value pairs";
                let tkn = if let Some(peek) = muncher.peek() {
                    format!("{:#?}", peek)
                } else {
                    "no token".into()
                };
                let (ln, col) = muncher.cursor_position();
                return Err(ParseTomlError::new(
                    msg.into(),
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
        };

        let text = SmolStr::new(&muncher.text()[s..e]);
        // println!("VALUE {:?}", text);
        Ok(Element::Node(Self {
            kind: Value,
            text,
            range: s..e,
            children,
        }))
    }

    fn inline_value(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, INLINE_ITEMS));
        muncher.reset_peek();
        match muncher.peek() {
            Some('"') => children.push(TomlNode::double_str(muncher)?),
            Some('t') | Some('f') => children.push(TomlToken::boolean(muncher)?),
            Some('[') => children.push(TomlNode::array(muncher)?),
            Some('{') => children.push(TomlNode::inline_table(muncher)?),
            Some(digi) if digi.is_numeric() => {
                muncher.reset_peek();
                let raw = muncher
                    .peek_until(|c| cmp_tokens(c, NUM_END))
                    .collect::<String>();
                if raw.contains(DATE_LIKE) {
                    children.push(TomlNode::date_time(muncher)?)
                } else if raw.contains('.') {
                    children.push(TomlNode::float(muncher)?)
                } else {
                    children.push(TomlToken::integer(muncher)?)
                }
            }
            None => children.push(Element::Token(TomlToken {
                kind: EoF,
                text: SmolStr::default(),
                range: 0..0,
            })),
            _ => {
                let msg = "invalid token in key value pairs";
                let tkn = if let Some(peek) = muncher.peek() {
                    format!("{:#?}", peek)
                } else {
                    "no token".into()
                };
                let (ln, col) = muncher.cursor_position();
                return Err(ParseTomlError::new(
                    msg.into(),
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
        };

        let text = SmolStr::new(&muncher.text()[s..e]);
        // println!("VALUE {:?}", text);
        Ok(Element::Node(Self {
            kind: Value,
            text,
            range: s..e,
            children,
        }))
    }

    fn key_value(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        if muncher.is_done() {
            println!("DONE in kv")
        }

        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, EOL));
        children.push(TomlNode::key(muncher)?);
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        children.push(TomlToken::equal(muncher)?);
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        children.push(TomlNode::value(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        // println!("{:?}", text);
        Ok(Element::Node(Self {
            kind: KeyValue,
            text,
            range: s..e,
            children,
        }))
    }

    fn inline_key_value(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        if muncher.is_done() {
            println!("DONE in kv")
        }

        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, INLINE_ITEMS));
        children.push(TomlNode::key(muncher)?);
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        children.push(TomlToken::equal(muncher)?);
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        children.push(TomlNode::inline_value(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        // println!("{:?}", text);
        Ok(Element::Node(Self {
            kind: KeyValue,
            text,
            range: s..e,
            children,
        }))
    }

    fn array_item(muncher: &mut Muncher) -> TomlResult<Option<Element>> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, ARRAY_ITEMS));
        
        children.push(TomlNode::value(muncher)?);
        if let Some(comma) = TomlToken::maybe_comma(muncher) {
            children.push(comma);
        }
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        if muncher.seek(2).map(|s| s.contains(']')) == Some(true) {
            return Ok(None);
        }

        let text = SmolStr::new(&muncher.text()[s..e]);
        Ok(Some(Element::Node(Self {
            kind: ArrayItem,
            text,
            range: s..e,
            children,
        })))
    }

    fn array(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        // TODO make a stack machine for this to count braces
        let (s, e) = muncher.peek_until_count(|c| c == &']');
        children.push(TomlToken::open_brace(muncher)?);

        while let Some(item) = TomlNode::array_item(muncher)? {
            children.push(item);
        }

        children.push(TomlToken::close_brace(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        Ok(Element::Node(Self {
            kind: Array,
            text,
            range: s..e,
            children,
        }))
    }

    fn inline_table(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }
        let start = muncher.position();

        children.push(TomlToken::open_curly(muncher)?);
        loop {
            // TODO this is weak make some sort of stack machine.
            if muncher.peek() == Some(&'}') {
                println!("BREAK");
                break;
            }
            // println!("{:?}", muncher.peek());
            children.push(TomlNode::inline_key_value(muncher)?);
            // an inline table and an array are the only two node types that
            // have comma's optionally eat comma and any following whitespace.
            if let Some(comma) = TomlToken::maybe_comma(muncher) {
                children.push(comma);
            }
            if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
                children.push(ws);
            }
        }
        println!("done with table");
        children.push(TomlToken::close_curly(muncher)?);
        let end = muncher.position();

        let text = SmolStr::new(&muncher.text()[start..end]);
        Ok(Element::Node(Self {
            kind: InlineTable,
            text,
            range: start..end,
            children,
        }))
    }

    fn heading(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        let (s, e) = muncher.peek_until_count(|c| c == &']');
        children.push(TomlToken::open_brace(muncher)?);

        match muncher.peek() {
            Some('"') => children.push(TomlNode::double_str(muncher)?),
            Some(ch) if ch.is_ascii() => children.push(TomlToken::ident_heading(muncher)?),
            Some(tkn) => {
                let (ln, col) = muncher.cursor_position();
                let msg = "invalid key token".into();
                let tkn = format!("{}", tkn);
                return Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
            None => println!("None in heading"),
        }

        children.push(TomlToken::close_brace(muncher)?);

        let text = SmolStr::new(&muncher.text()[s..e]);
        Ok(Element::Node(Self {
            kind: Heading,
            text,
            range: s..e,
            children,
        }))
    }

    fn table(muncher: &mut Muncher) -> TomlResult<Element> {
        let mut children = Vec::default();

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            children.push(ws);
        }

        let start = muncher.position();

        children.push(TomlNode::heading(muncher)?);
        loop {
            if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
                children.push(ws);
            }
            // TODO this is weak.
            if muncher.is_done() {
                break;
            }
            children.push(TomlNode::key_value(muncher)?);
        }

        let end = muncher.position();
        let text = SmolStr::new(&muncher.text()[start..end]);
        Ok(Element::Node(Self {
            kind: Table,
            text,
            range: start..end,
            children,
        }))
    }
}

impl Element {
    fn root(muncher: &mut Muncher) -> TomlResult<Element> {
        let text = SmolStr::new(muncher.text());
        let children = Tokenizer::parse_file(muncher)?;
        let end = text.len();
        Ok(Element::Node(TomlNode {
            kind: Root,
            text,
            range: 0..end,
            children,
        }))
    }
}

pub struct Tokenizer {
    ast: Vec<Element>,
}

impl Tokenizer {
    pub fn parse(input: &str) -> TomlResult<Element> {
        let mut muncher = Muncher::new(input);
        Ok(Element::root(&mut muncher)?)
    }

    /// It seems the only two top level Elements are key value pairs,
    /// tables and comments
    fn parse_file(muncher: &mut Muncher) -> TomlResult<Vec<Element>> {
        let mut elements = Vec::default();

        loop {
            if muncher.is_done() { break; }
            println!("MAIN LOOP");
            match muncher.peek() {
                Some('#') => {
                    let cmt = TomlNode::comment(muncher)?;
                    elements.push(cmt);
                }
                Some('[') => {
                    let table = TomlNode::table(muncher)?;
                    elements.push(table);
                }
                Some(ch) if ch.is_ascii() => {
                    println!("{:?}", ch);
                    let kv = TomlNode::key_value(muncher)?;
                    elements.push(kv);
                }
                Some(tkn) => {
                    let msg = "toml file must be key values or tables".into();
                    let tkn = format!("{}", tkn);
                    let (ln, col) = muncher.cursor_position();
                    return Err(ParseTomlError::new(
                        msg,
                        TomlErrorKind::UnexpectedToken { tkn, ln, col },
                    ));
                }
                None => {
                    let end = muncher.position();
                    elements.push(Element::Token(TomlToken {
                        kind: EoF,
                        text: SmolStr::default(),
                        range: end..end,
                    }));
                    break;
                }
            }
        }
        Ok(elements)
    }
}

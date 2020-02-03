use rowan::SmolStr;

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};
use super::kinds::TomlKind::{self, *};

use chrono::{NaiveDate, NaiveTime};

use super::kinds::{Element, TomlNode, TomlToken};
use super::munch::{
    cmp_tokens, Muncher, ARRAY_ITEMS, BOOL_END, DATE_CHAR, DATE_END, DATE_LIKE, DATE_TIME, EOL,
    IDENT_END, INLINE_ITEMS, INT_END, KEY_END, NUM_END, SEG_END, TIME_CHAR, WHITESPACE,
};
use super::syntax::Parser;

impl Into<(TomlKind, SmolStr)> for Element {
    fn into(self) -> (TomlKind, SmolStr) {
        match self {
            Element::Node(n) => (n.kind, n.text),
            Element::Token(tkn) => (tkn.kind, tkn.text),
        }
    }
}

fn is_valid_key(s: &str) -> bool {
    s.chars().all(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '-' | '\'' | '"' => true,
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
                NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?).and_hms(
                    time[0].parse()?,
                    time[1].parse()?,
                    time[2].parse()?,
                )
            };
        Ok(true)
    }
}

impl TomlToken {
    // fn whitespace(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
    //     let (s, e) = muncher.eat_until_count(|c| !cmp_tokens(c, WHITESPACE));
    //     // TODO is this more efficient than eat_until to String??
    //     let text = SmolStr::new(&muncher.text()[s..e]);
    //     parser.builder.token(Whitespace.into(), text);
    //     Ok(())
    // }

    /// Returns Element if whitespace was found.
    fn maybe_whitespace(muncher: &mut Muncher) -> Option<Element> {
        let (s, e) = muncher.eat_until_count(|c| !cmp_tokens(c, WHITESPACE));
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        if e > s {
            Some(Element::Token(Self {
                kind: Whitespace,
                text,
            }))
        } else {
            None
        }
    }

    fn hash(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_hash());
        parser.builder.token(Hash.into(), SmolStr::new("#"));
        Ok(())
    }

    // fn plus(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
    //     assert!(muncher.eat_plus());
    //     parser.builder.token(Plus.into(), SmolStr::new("+"));
    //     Ok(())
    // }

    // fn minus(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
    //     assert!(muncher.eat_minus());
    //     parser.builder.token(Minus.into(), SmolStr::new("-"));
    //     Ok(())
    // }

    fn equal(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_eq());
        parser.builder.token(Equal.into(), SmolStr::new("="));
        Ok(())
    }

    // fn comma(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
    //     assert!(muncher.eat_comma());
    //     parser.builder.token(Comma.into(), SmolStr::new(","));
    //     Ok(())
    // }

    /// Returns Element if comma was found. The last item
    /// in an array may or may not have a comma.
    fn maybe_comma(muncher: &mut Muncher) -> Option<Element> {
        let start = muncher.position();
        if muncher.eat_comma() {
            let end = start + 1;
            Some(Element::Token(Self {
                kind: Comma,
                text: SmolStr::new(","),
            }))
        } else {
            None
        }
    }

    fn colon(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_colon());
        parser.builder.token(Colon.into(), SmolStr::new(":"));
        Ok(())
    }

    fn dot(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_dot());
        parser.builder.token(Dot.into(), SmolStr::new("."));
        Ok(())
    }

    fn maybe_dot(muncher: &mut Muncher) -> Option<Element> {
        let start = muncher.position();
        if muncher.eat_dot() {
            let end = start + 1;
            Some(Element::Token(Self {
                kind: Dot,
                text: SmolStr::new("."),
            }))
        } else {
            None
        }
    }

    fn double_quote(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_double_quote());
        parser.builder.token(DoubleQuote.into(), SmolStr::new("\""));
        Ok(())
    }

    fn triple_quote(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_double_quote());
        assert!(muncher.eat_double_quote());
        assert!(muncher.eat_double_quote());
        parser
            .builder
            .token(TripleQuote.into(), SmolStr::new("\"\"\""));
        Ok(())
    }

    fn single_quote(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_single_quote());
        parser.builder.token(SingleQuote.into(), SmolStr::new("\'"));
        Ok(())
    }

    fn ident(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, IDENT_END));
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        parser.builder.token(Ident.into(), text);
        Ok(())
    }

    fn ident_seg(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, SEG_END));
        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("{:?}", text);
        parser.builder.token(Ident.into(), text);
        Ok(())
    }

    fn ident_double_str(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| c == &'"');
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);
        parser.builder.token(Ident.into(), text);
        Ok(())
    }

    fn ident_triple_str(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_range_of("\"\"\"");
        println!("{} {}", muncher.text().chars().count(), e);
        let text = SmolStr::new(&muncher.text()[s..e]);
        parser.builder.token(Ident.into(), text);
        Ok(())
    }

    fn ident_single_str(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| c == &'\'');
        let text = SmolStr::new(&muncher.text()[s..e]);
        parser.builder.token(Ident.into(), text);
        Ok(())
    }

    fn comment_text(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, EOL));
        let text = SmolStr::new(&muncher.text()[s..e]);
        parser.builder.token(CommentText.into(), text);
        Ok(())
    }

    fn open_brace(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_open_brc());
        parser.builder.token(OpenBrace.into(), SmolStr::new("["));
        Ok(())
    }
    fn close_brace(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_close_brc());
        parser.builder.token(CloseBrace.into(), SmolStr::new("]"));
        Ok(())
    }
    fn open_curly(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_open_curly());
        parser.builder.token(OpenCurly.into(), SmolStr::new("{"));
        Ok(())
    }
    fn close_curly(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        assert!(muncher.eat_close_curly());
        parser.builder.token(CloseCurly.into(), SmolStr::new("}"));
        Ok(())
    }

    fn boolean(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, BOOL_END));
        let boolean = &muncher.text()[s..e];

        let text = SmolStr::new(boolean);
        if boolean == "true" || boolean == "false" {
            parser.builder.token(Bool.into(), text);
            Ok(())
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

    fn integer(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, INT_END));
        let int = &muncher.text()[s..e];

        if int.chars().all(|c| c.is_numeric()) {
            let text = SmolStr::new(int);
            parser.builder.token(Integer.into(), text);
            Ok(())
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
    fn comment(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Comment.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        TomlToken::hash(muncher, parser)?;
        TomlToken::comment_text(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Whitespace`, `Integer`, `Dot` and `Integer token and adds them as
    /// children.
    fn float(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Float.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        TomlToken::integer(muncher, parser)?;
        TomlToken::dot(muncher, parser)?;
        TomlToken::integer(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Date` node from `Whitespace` and `Date` token and if valid adds them as
    /// children.
    fn date_time(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Date.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        let (s, e) = muncher.eat_until_count(|c| cmp_tokens(c, DATE_END));

        let text = SmolStr::new(&muncher.text()[s..e]);

        if is_valid_datetime(&text) != Ok(true) {
            let (ln, col) = muncher.cursor_position();
            let msg = "invalid integer".into();
            Err(ParseTomlError::new(
                msg,
                TomlErrorKind::UnexpectedToken {
                    tkn: text.into(),
                    ln,
                    col,
                },
            ))
        } else {
            parser.builder.token(Ident.into(), text);
            parser.builder.finish_node();
            Ok(())
        }
    }

    /// Builds `Str` node from `Whitespace`, `SingleQuote` and `Ident` token and adds them as
    /// children.
    fn single_str(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Str.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlToken::single_quote(muncher, parser)?;
        TomlToken::ident_single_str(muncher, parser)?;
        TomlToken::single_quote(muncher, parser)?;

        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Str` node from `Whitespace`, `DoubleQuote` and `Ident` token and adds them as
    /// children.
    fn double_str(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Str.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlToken::double_quote(muncher, parser)?;
        TomlToken::ident_double_str(muncher, parser)?;
        TomlToken::double_quote(muncher, parser)?;

        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Str` node from `Whitespace`, `DoubleQuote` and `Ident` token and adds them as
    /// children.
    fn string(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Str.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        if muncher.seek(2).map(|s| s == "\"\"") == Some(true) {
            println!("TRIPLE");
            TomlToken::triple_quote(muncher, parser)?;
            TomlToken::ident_triple_str(muncher, parser)?;
            TomlToken::triple_quote(muncher, parser)?;
        } else {
            TomlToken::double_quote(muncher, parser)?;
            TomlToken::ident_double_str(muncher, parser)?;
            TomlToken::double_quote(muncher, parser)?;
        }

        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Key` node from `Whitespace` and `Ident` token and adds them as
    /// children.
    fn key(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Key.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        let (s, e) = muncher.peek_until_count(|c| cmp_tokens(c, KEY_END));
        // muncher.reset_peek();
        match muncher.peek() {
            Some(&'"') => TomlNode::double_str(muncher, parser),
            Some(&'\'') => TomlNode::single_str(muncher, parser),
            Some(ch) if ch.is_ascii() => TomlToken::ident(muncher, parser),
            Some(tkn) => {
                let (ln, col) = muncher.cursor_position();
                let msg = "invalid key token".into();
                let tkn = format!("{}", tkn);
                return Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
            None => todo!("NONE in key"),
        }?;

        let text = SmolStr::new(&muncher.text()[s..e]);
        println!("{:?}", text);
        if is_valid_key(&text) {
            parser.builder.finish_node();
            Ok(())
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

    /// Builds `Value` node from `Whitespace` and whatever value node is present
    /// and adds them as children. this is called for top level key value pairs
    /// and tables.
    fn value(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Value.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        match muncher.peek() {
            Some('"') => TomlNode::string(muncher, parser),
            Some('\'') => TomlNode::single_str(muncher, parser),
            Some('t') | Some('f') => TomlToken::boolean(muncher, parser),
            Some('[') => TomlNode::array(muncher, parser),
            Some('{') => TomlNode::inline_table(muncher, parser),
            Some(digi) if digi.is_numeric() => {
                muncher.reset_peek();
                let raw = muncher
                    .peek_until(|c| cmp_tokens(c, NUM_END))
                    .collect::<String>();
                if raw.contains(DATE_LIKE) {
                    TomlNode::date_time(muncher, parser)
                } else if raw.contains('.') {
                    TomlNode::float(muncher, parser)
                } else {
                    TomlToken::integer(muncher, parser)
                }
            }
            None => unimplemented!("found EOF in value"),
            _ => {
                let msg = "invalid token in value";
                let tkn = if let Some(peek) = muncher.peek() {
                    format!("{:?}", peek)
                } else {
                    "no token".into()
                };
                let (ln, col) = muncher.cursor_position();
                return Err(ParseTomlError::new(
                    msg.into(),
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
        }?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Value` node from `Whitespace` and whatever value node is present
    /// and adds them as children. This is called for inline tables only.
    fn inline_value(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Value.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        match muncher.peek() {
            Some('"') => TomlNode::string(muncher, parser),
            Some('\'') => TomlNode::single_str(muncher, parser),
            Some('t') | Some('f') => TomlToken::boolean(muncher, parser),
            Some('[') => TomlNode::array(muncher, parser),
            Some('{') => TomlNode::inline_table(muncher, parser),
            Some(digi) if digi.is_numeric() => {
                muncher.reset_peek();
                let raw = muncher
                    .peek_until(|c| cmp_tokens(c, NUM_END))
                    .collect::<String>();
                if raw.contains(DATE_LIKE) {
                    TomlNode::date_time(muncher, parser)
                } else if raw.contains('.') {
                    TomlNode::float(muncher, parser)
                } else {
                    TomlToken::integer(muncher, parser)
                }
            }
            None => unimplemented!("value found EOF"),
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
        }?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `KeyValue` node from `Whitespace`, `Key` and whatever value node is present
    /// and adds them as children.
    fn key_value(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        if muncher.is_done() {
            todo!("DONE in kv")
        }
        parser.builder.start_node(KeyValue.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        if muncher.peek() == Some(&'#') {
            TomlNode::comment(muncher, parser)?;
            parser.builder.finish_node();
            return Ok(());
        }

        TomlNode::key(muncher, parser)?;

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlToken::equal(muncher, parser)?;

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlNode::value(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `KeyValue` node from `Whitespace`, `Key` and whatever value node is present
    /// and adds them as children. This is only for `InlineTable`s.
    fn inline_key_value(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        if muncher.is_done() {
            todo!("DONE in kv")
        }
        parser.builder.start_node(KeyValue.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlNode::key(muncher, parser)?;
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlToken::equal(muncher, parser)?;
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }
        TomlNode::inline_value(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `ArrayItem` node from `Whitespace` and whatever `Value` node is present
    /// and adds them as children.
    fn array_item(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<Option<()>> {
        if muncher.peek() == Some(&']') {
            return Ok(None);
        }

        parser.builder.start_node(ArrayItem.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlNode::value(muncher, parser)?;

        if let Some(comma) = TomlToken::maybe_comma(muncher) {
            let (kind, text) = comma.into();
            parser.builder.token(kind.into(), text);
        }
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text);
        }

        parser.builder.finish_node();
        Ok(Some(()))
    }

    /// Builds `Array` node from `Whitespace` and whatever `ArrayItem` nodes are present
    /// and adds them as children.
    fn array(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Array.into());
        TomlToken::open_brace(muncher, parser)?;
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        while let Some(_) = TomlNode::array_item(muncher, parser)? { /* loop to array end */ }

        TomlToken::close_brace(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `InlineTable` node from `Whitespace` and whatever `KeyValue` nodes are present
    /// and adds them as children.
    fn inline_table(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(InlineTable.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlToken::open_curly(muncher, parser)?;
        loop {
            // TODO this is weak make some sort of stack machine.
            if muncher.peek() == Some(&'}') {
                break;
            }
            // println!("{:?}", muncher.peek());
            TomlNode::inline_key_value(muncher, parser)?;
            // an inline table and an array are the only two node types that
            // have comma's optionally eat comma and any following whitespace.
            if let Some(comma) = TomlToken::maybe_comma(muncher) {
                let (kind, text) = comma.into();
                parser.builder.token(kind.into(), text)
            }
            if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
                let (kind, text) = ws.into();
                parser.builder.token(kind.into(), text)
            }
        }
        TomlToken::close_curly(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// If `Heading` contains a '.' a `TomlNode` `SegIdent` is produced other wise
    /// a plain `TomlToken` `Ident` is added.
    fn ident_heading(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        let (s, e) = muncher.peek_until_count(|c| c == &']');
        // TODO is this more efficient than eat_until to String??
        let text = SmolStr::new(&muncher.text()[s..e]);

        if text.contains('"') {
            parser.builder.start_node(SegIdent.into());
            if text.contains('.') {
                let mut txt = text.clone();
                loop {
                    // TODO if not in loop `value moved here, in previous iteration of loop` error BOOO
                    let mut in_str = false;
                    let dot_index = |ch: char| -> bool {
                        if ch == '"' && !in_str {
                            in_str = true;
                        } else if ch == '"' && in_str {
                            in_str = false;
                        };

                        ch == '.' && !in_str
                    };
                    if txt.starts_with('"') {
                        TomlNode::double_str(muncher, parser)?;
                        if let Some(dot) = TomlToken::maybe_dot(muncher) {
                            let (kind, text) = dot.into();
                            parser.builder.token(kind.into(), text)
                        }
                        if let Some(idx) = txt.chars().position(dot_index) {
                            println!("{:?}", txt.split_at(idx));
                            txt = SmolStr::from(txt.split_at(idx).1);
                        } else {
                            break;
                        }
                    } else {
                        TomlToken::ident_seg(muncher, parser)?;
                        if let Some(dot) = TomlToken::maybe_dot(muncher) {
                            let (kind, text) = dot.into();
                            parser.builder.token(kind.into(), text)
                        }
                        if let Some(idx) = txt.chars().position(dot_index) {
                            println!("{:?}", txt.split_at(idx));
                            txt = SmolStr::from(txt.split_at(idx + 1).1);
                        } else {
                            break;
                        }
                    }
                }
            } else {
                TomlNode::double_str(muncher, parser)?;
            }
            parser.builder.finish_node();
            Ok(())
        } else if text.contains('.') {
            parser.builder.start_node(SegIdent.into());
            println!("DOT {:?}", text);

            TomlToken::ident_seg(muncher, parser)?;
            TomlToken::dot(muncher, parser)?;
            TomlToken::ident_seg(muncher, parser)?;

            // for all segments after the first we loop for each
            for _ in 2..text.split('.').count() {
                if let Some(dot) = TomlToken::maybe_dot(muncher) {
                    let (kind, text) = dot.into();
                    parser.builder.token(kind.into(), text);
                    TomlToken::ident_seg(muncher, parser)?;
                }
            }
            parser.builder.finish_node();
            Ok(())
        } else {
            parser.builder.token(Ident.into(), text);
            Ok(())
        }
    }

    /// Builds `Heading` node from `Whitespace` and either `Ident` token or
    /// `SegIdent` node and adds them as children.
    fn heading(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Heading.into());

        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlToken::open_brace(muncher, parser)?;

        match muncher.peek() {
            // Some('"') => TomlNode::double_str(muncher, parser)?,
            Some(ch) if ch.is_ascii() => TomlNode::ident_heading(muncher, parser)?,
            Some(tkn) => {
                let (ln, col) = muncher.cursor_position();
                let msg = "invalid heading token".into();
                let tkn = format!("{}", tkn);
                return Err(ParseTomlError::new(
                    msg,
                    TomlErrorKind::UnexpectedToken { tkn, ln, col },
                ));
            }
            None => todo!("heading NONE"),
        };
        // although this is an iterator it advances the cursor
        let _eaten = muncher.eat_until(|c| c == &']');

        TomlToken::close_brace(muncher, parser)?;
        parser.builder.finish_node();
        Ok(())
    }

    /// Builds `Table` node from `Whitespace` and whatever `KeyValue` nodes are present
    /// and adds them as children.
    fn table(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Table.into());
        if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
            let (kind, text) = ws.into();
            parser.builder.token(kind.into(), text)
        }

        TomlNode::heading(muncher, parser)?;
        loop {
            if muncher.seek(4).map(|s| s.contains('[')) == Some(true) {
                break;
            }
            if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
                let (kind, text) = ws.into();
                parser.builder.token(kind.into(), text)
            }
            // TODO this is weak.
            if muncher.is_done() {
                break;
            }
            TomlNode::key_value(muncher, parser)?;
        }
        parser.builder.finish_node();
        Ok(())
    }
}

pub struct Tokenizer;

impl Tokenizer {
    /// Returns a wrapper around a `rowan::GreenNodeBuilder` called `Parser`.
    /// The can be turned into a walk-able `SyntaxNode`.
    ///
    /// # Examples
    /// ```
    /// # use toml_parse::{Tokenizer, Parser};
    /// # use rowan::GreenNodeBuilder;
    /// let toml = "";
    /// let parse_builder = Parser::new();
    /// let parsed = Tokenizer::parse(toml, parse_builder).expect("parse failed");
    /// let green_node = parsed.parse().expect("parse failed");
    /// let root_node = green_node.syntax();
    /// ```
    pub fn parse(input: &str, mut p: Parser) -> TomlResult<Parser> {
        let mut muncher = Muncher::new(input);
        Tokenizer::parse_file(&mut muncher, &mut p)?;
        Ok(p)
    }

    /// It seems the only three top level Kinds are `KeyValue` pairs,
    /// `Table`s and `Comments`.
    fn parse_file(muncher: &mut Muncher, parser: &mut Parser) -> TomlResult<()> {
        parser.builder.start_node(Root.into());
        // let text = SmolStr::new(muncher.text());
        // let end = text.len();
        loop {
            if let Some(ws) = TomlToken::maybe_whitespace(muncher) {
                let (kind, text) = ws.into();
                parser.builder.token(kind.into(), text)
            }
            if muncher.is_done() {
                parser.builder.token(EoF.into(), SmolStr::default());
                break;
            }

            match muncher.peek() {
                Some('#') => {
                    TomlNode::comment(muncher, parser)?;
                }
                Some('[') => {
                    TomlNode::table(muncher, parser)?;
                }
                Some(ch) if ch.is_ascii() => {
                    TomlNode::key_value(muncher, parser)?;
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
                    parser.builder.token(EoF.into(), SmolStr::default());
                    break;
                }
            }
        }
        parser.builder.finish_node();
        Ok(())
    }
}

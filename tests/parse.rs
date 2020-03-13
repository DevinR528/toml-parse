use std::fs::read_to_string;

use toml_parse::{parse_it, SyntaxNodeExtTrait};

#[test]
fn parse_table_comment() {
    let file = "[table]\n# hello there";
    let parsed = parse_it(file).expect("parse failed");
    let root = parsed.syntax();
    assert_eq!(root.token_text(), file)
}

#[test]
fn parse_single_quote_key() {
    let file = "[table]\n'key' = \"value\"";
    let parsed = parse_it(file).expect("parse failed");
    let root = parsed.syntax();
    assert_eq!(root.token_text(), file)
}

#[test]
fn parse_double_quote_key() {
    let file = "[table]\n\"key\" = \"value\"";
    let parsed = parse_it(file).expect("parse failed");
    let root = parsed.syntax();
    assert_eq!(root.token_text(), file)
}

#[test]
fn parse_single_quote_value() {
    let file = "[table]\nkey = 'value'";
    let parsed = parse_it(file).expect("parse failed");
    let root = parsed.syntax();
    assert_eq!(root.token_text(), file)
}

#[test]
fn parse_triple_quote_value() {
    let file = "[table]\nkey = \"\"\"value\"\"\"";
    let parsed = parse_it(file).expect("parse failed");
    let root = parsed.syntax();
    assert_eq!(root.token_text(), file)
}

#[test]
fn parse_triple_quote_value_complex() {
    let file = "[table]\nkey = \"\"\"value \"hello\" bye\n end\"\"\"";
    let parsed = parse_it(file).expect("parse failed");
    let root = parsed.syntax();
    assert_eq!(root.token_text(), file)
}

#[test]
fn parse_all_tokens() {
    let file = r#"[deps]
alpha = "beta"
number = 1234
array = [ true, false, true ]
inline-table = { date = 1988-02-03T10:32:10, }
"#;
    let parsed = parse_it(file).expect("parse failed");
    println!("{:#?}", parsed.syntax());
    assert_eq!(parsed.syntax().token_text(), file)
}

#[test]
fn parse_ftop_file() {
    let input = read_to_string("examp/ftop.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed");
    assert_eq!(parsed.syntax().token_text(), input)
}
#[test]
fn parse_fend_file() {
    let input = read_to_string("examp/fend.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed");
    assert_eq!(parsed.syntax().token_text(), input)
}
#[test]
fn parse_seg_file() {
    let input = read_to_string("examp/seg.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed");
    assert_eq!(parsed.syntax().token_text(), input)
}
#[test]
fn parse_work_file() {
    let input = read_to_string("examp/work.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed");
    println!("{:#?}", parsed.syntax());
    assert_eq!(parsed.syntax().token_text(), input)
}

#[test]
fn parse_print_token_text() {
    let input = read_to_string("examp/seg_sort.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed");
    // println!("{:#?}", parsed)
    assert_eq!(parsed.syntax().token_text(), input)
}

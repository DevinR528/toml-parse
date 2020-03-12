use std::fs::read_to_string;

use toml_parse::{parse_it, sort_toml_items, Formatter, Matcher, SyntaxNodeExtTrait, TomlKind};

const HEADERS: [&str; 3] = [
    "[dependencies]",
    "[dev-dependencies]",
    "[build-dependencies]",
];

const HEADER_SEG: [&str; 3] = ["dependencies.", "dev-dependencies.", "build-dependencies."];

const MATCHER: Matcher<'_> = Matcher {
    heading: &HEADERS,
    segmented: &HEADER_SEG,
    heading_key: &[("[workspace]", "members"), ("[workspace]", "exclude")],
};

#[test]
fn sort_fmt_comment_tkns() {
    let file = r#"# comment
[dependencies]
number = 1234
# comment
alpha = "beta"
"#;
    let parsed = parse_it(file).expect("parse failed").syntax();
    let parsed2 = parse_it(file).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_ne!(file, Formatter::new(&sorted).format().to_string())
}

#[test]
fn sort_fmt_seg_sort() {
    let input = read_to_string("examp/seg_sort.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    print!("{}", Formatter::new(&sorted).format().to_string());
    print!("{}", sorted.token_text());

    assert_ne!(input, Formatter::new(&sorted).format().to_string())
}

#[test]
fn sort_fmt_seg_sort_ok() {
    let input = read_to_string("examp/seg_sort_ok.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    // NO SORTING
    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_eq!(input, Formatter::new(&sorted).format().to_string())
}

#[test]
fn sort_fmt_seg() {
    let input = read_to_string("examp/seg.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    // NO SORTING
    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_eq!(input, Formatter::new(&sorted).format().to_string())
}

#[test]
fn sort_fmt_fend() {
    let input = read_to_string("examp/fend.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_ftop() {
    let input = read_to_string("examp/ftop.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    print!("{:#?}", parsed);

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);
    print!("{}", sorted.token_text());

    // assert!(!parsed.deep_eq(&sorted));
    // assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_obj_comma() {
    let input = read_to_string("examp/obj_comma.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();
    // REMOVED \n\n AFTER HEADING
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_cmt_eol() {
    let input = read_to_string("examp/test.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_win() {
    let input = read_to_string("examp/win.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_work() {
    let input = read_to_string("examp/work.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();

    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_indent_work() {
    let input = read_to_string("examp/indent.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();

    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

#[test]
fn sort_fmt_right_seg_header() {
    let input = read_to_string("examp/right.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &MATCHER);

    print!("{}", sorted.token_text());

    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().to_string();

    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().to_string())
}

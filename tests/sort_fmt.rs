use std::fs::read_to_string;

use toml_parse::{walk, parse_it, sort_toml_items, Matcher, SyntaxNodeExtTrait, TomlKind, Formatter};

const HEADER: Matcher<'static> = Matcher {
    heading: &["[dependencies]"],
    segmented: &["dependencies."],
    heading_key: &[("[workspace]", "members")],
    value: TomlKind::Array,
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

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_ne!(file, Formatter::new(&sorted).format().formatted)
}

#[test]
fn sort_fmt_seg_sort() {
    let input = read_to_string("examp/seg_sort.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_ne!(input, Formatter::new(&sorted).format().formatted)
}

#[test]
fn sort_fmt_seg_sort_ok() {
    let input = read_to_string("examp/seg_sort_ok.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);
    
    // NO SORTING
    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_eq!(input, Formatter::new(&sorted).format().formatted)
}

#[test]
fn sort_fmt_seg() {
    let input = read_to_string("examp/seg.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    // NO SORTING
    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    assert_eq!(input, Formatter::new(&sorted).format().formatted)
}

#[test]
fn sort_fmt_fend() {
    let input = read_to_string("examp/fend.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

#[test]
fn sort_fmt_ftop() {
    let input = read_to_string("examp/ftop.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

#[test]
fn sort_fmt_obj_comma() {
    let input = read_to_string("examp/obj_comma.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;
    // REMOVED \n\n AFTER HEADING
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

#[test]
fn sort_fmt_cmt_eol() {
    let input = read_to_string("examp/test.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

#[test]
fn sort_fmt_win() {
    let input = read_to_string("examp/win.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;
    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

#[test]
fn sort_fmt_work() {
    let input = read_to_string("examp/work.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;

    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

#[test]
fn sort_fmt_indent_work() {
    let input = read_to_string("examp/indent.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());

    let fmted = Formatter::new(&sorted).format().formatted;

    assert_ne!(input, fmted);

    let idempotent = parse_it(&fmted).expect("parse failed").syntax();
    assert_eq!(fmted, Formatter::new(&idempotent).format().formatted)
}

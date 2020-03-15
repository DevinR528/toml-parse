use std::fs::read_to_string;

use toml_parse::{parse_it, sort_toml_items, walk, Matcher, SyntaxNode, SyntaxNodeExtTrait};

const HEADER: Matcher<'static> = Matcher {
    heading: &["[dependencies]"],
    segmented: &["dependencies."],
    heading_key: &[("[workspace]", "members")],
};

#[allow(dead_code)]
fn print_overlaping(sorted: &SyntaxNode, parsed: &SyntaxNode) {
    for (p, s) in walk(parsed).zip(walk(sorted)) {
        println!("PARSED={:?} SORTED={:?}", p, s);
    }
}
#[test]
fn comment_tkns() {
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
}

#[test]
fn sort_tkns_ftop() {
    let input = read_to_string("examp/ftop.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

#[test]
fn sort_tkns_seg() {
    let input = read_to_string("examp/seg_sort.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));
    // println!("{}", parsed.token_text());

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

#[test]
fn sort_tkns_seg_ok() {
    let input = read_to_string("examp/seg_sort_ok.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

#[test]
fn sort_tkns_work() {
    let input = read_to_string("examp/work.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}
#[test]
fn sort_tkns_fend() {
    let input = read_to_string("examp/fend.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

#[test]
fn sort_tkns_right() {
    let input = read_to_string("examp/right.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);

    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

#[test]
fn sort_tkns_win() {
    let input = read_to_string("examp/win.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);
    // println!("{}", sorted.token_text());

    assert!(!parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

#[test]
fn sort_tkns_seg_sort_ok() {
    let input = read_to_string("examp/seg_sort_ok.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let parsed2 = parse_it(&input).expect("parse failed").syntax();

    assert!(parsed.deep_eq(&parsed2));

    let sorted = sort_toml_items(&parsed, &HEADER);
    // println!("{}", sorted.token_text());

    assert!(parsed.deep_eq(&sorted));
    assert_eq!(sorted.text_range(), parsed.text_range());
}

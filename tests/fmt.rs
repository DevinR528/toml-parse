use std::fs::read_to_string;

use toml_parse::{parse_it, Formatter, RuleConfig};

#[test]
fn fmt_eq_space() {
    let file = "[table]\nkey=false";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmted = Formatter::new(&parsed).format();
    assert_eq!("[table]\nkey = false\n", fmted.to_string());
}
#[test]
fn fmt_eq_space_many() {
    let file = "[table]\nkey   =  false\n";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmted = Formatter::new(&parsed).format();
    assert_eq!("[table]\nkey = false\n", fmted.to_string());
}
#[test]
fn fmt_heading() {
    let file = "[table] key = false";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new(&parsed).format();
    assert_eq!(fmt.to_string(), "[table]\nkey = false\n")
}
#[test]
fn fmt_comma_arr() {
    let file = "key = [1,2,3]";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new(&parsed).format();
    assert_eq!(fmt.to_string(), "key = [ 1, 2, 3 ]\n")
}

#[test]
fn fmt_array_no_space() {
    let file = "key = [1,2,3]";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new_with_config(
        &parsed,
        RuleConfig {
            space_before_after_array_brace: false,
            space_before_after_curly_brace: true,
        },
    )
    .format();
    assert_eq!(fmt.to_string(), "key = [1, 2, 3]\n");

    // still works the same
    let file = "key={a=1,b=2}";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new_with_config(
        &parsed,
        RuleConfig {
            space_before_after_array_brace: false,
            space_before_after_curly_brace: true,
        },
    )
    .format();
    assert_eq!(fmt.to_string(), "key = { a = 1, b = 2 }\n");

    let fmt = Formatter::new_with_config(
        &parsed,
        RuleConfig {
            space_before_after_array_brace: false,
            space_before_after_curly_brace: false,
        },
    )
    .format();
    assert_eq!(fmt.to_string(), "key = {a = 1, b = 2}\n")
}

#[test]
fn fmt_comma_obj() {
    let file = "key={a=1,b=2}";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new(&parsed).format();
    assert_eq!(fmt.to_string(), "key = { a = 1, b = 2 }\n")
}
#[test]
fn fmt_tables() {
    let file = "[table]\nkey = false [table]\nkey = 1";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new(&parsed).format();
    assert_eq!(
        fmt.to_string(),
        "[table]\nkey = false\n\n[table]\nkey = 1\n"
    )
}
#[test]
fn fmt_seg() {
    let file = "[table . more]\nkey = false\n";
    let parsed = parse_it(file).expect("parse failed").syntax();
    let fmt = Formatter::new(&parsed).format();
    assert_eq!(fmt.to_string(), "[table.more]\nkey = false\n")
}
#[test]
fn fmt_indent_arr() {
    let input = read_to_string("examp/indent.toml").expect("file read failed");
    let fixed = read_to_string("examp/indent.fix.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();
    let fmted = Formatter::new(&parsed).format();

    assert_eq!(fmted.to_string(), fixed);
}
#[test]
fn fmt_obj_comma() {
    let input = read_to_string("examp/obj_comma.toml").expect("file read failed");
    let fixed = read_to_string("examp/obj_comma.fix.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();

    let fmted = Formatter::new(&parsed).format();

    assert_eq!(fmted.to_string(), fixed);
}

#[test]
fn fmt_array_table() {
    let input = read_to_string("examp/arr_table.toml").expect("file read failed");
    let fixed = read_to_string("examp/arr_table.fix.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();

    let fmted = Formatter::new(&parsed).format();

    assert_eq!(fmted.to_string(), fixed);
}

#[test]
fn fmt_clippy() {
    let input = read_to_string("examp/clippy.toml").expect("file read failed");
    let fixed = read_to_string("examp/clippy.fix.toml").expect("file read failed");
    let parsed = parse_it(&input).expect("parse failed").syntax();

    let fmted = Formatter::new(&parsed).format();

    assert_eq!(fmted.to_string(), fixed);
}

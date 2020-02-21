use std::fmt;

pub(self) use super::tkn_tree::{self, walk::walk_tokens_non_ws, SyntaxNode, TomlKind};

mod block;
mod rules;
mod ws;

use block::Block;
use rules::{
    indent_after_comma, lf_after_heading, lf_after_table, none_around_dot, space_around_eq,
    space_lf_after_array_open, space_lf_after_comma, space_lf_after_inline_table_open,
    space_lf_before_array_close, space_lf_before_inline_table_close,
};
use ws::WhiteSpace;

type RuleFn = Box<dyn for<'a> Fn(&'a Block, &'a Block) -> Option<WhiteSpace>>;

pub struct Formatter {
    pub blocks: Vec<Block>,
    pub rules: Vec<(TomlKind, RuleFn)>,
    pub formatted: String,
}

impl Formatter {
    pub fn new(root: &SyntaxNode) -> Formatter {
        Self {
            blocks: walk_tokens_non_ws(root).map(Block::new).collect(),
            rules: formatter(),
            formatted: String::default(),
        }
    }

    pub fn format(mut self) -> Self {
        let zipped = self.blocks.iter().zip(self.blocks.iter().skip(1));

        for (l_blk, r_blk) in zipped {
            let rules = self
                .rules
                .iter()
                .filter(|(kind, _)| *kind == l_blk.kind())
                .chain(self.rules.iter().filter(|(kind, _)| *kind == r_blk.kind()))
                .map(|(_, func)| func);

            for rule in rules {
                if let Some(fixed) = rule(l_blk, r_blk) {
                    println!("{:#?}", fixed);
                    r_blk.whitespace.set(fixed);
                }
            }
        }
        self.formatted = self
            .blocks
            .clone()
            .into_iter()
            .map(|b| b.to_string())
            .collect();
        if !self.formatted.ends_with('\n') {
            self.formatted.push('\n')
        }
        self
    }
}
impl fmt::Debug for Formatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Formatter")
            .field("blocks", &self.blocks)
            .field(
                "rules",
                &self.rules.iter().map(|(k, _fn)| k).collect::<Vec<_>>(),
            )
            .field("formatted", &self.formatted)
            .finish()
    }
}

pub fn formatter() -> Vec<(TomlKind, RuleFn)> {
    vec![
        // WHITESPACE
        // empty line between tables
        (TomlKind::OpenBrace, Box::new(lf_after_table) as RuleFn),
        // newline after heading before key values
        (TomlKind::CloseBrace, Box::new(lf_after_heading) as RuleFn),
        // nothing around dot in heading
        (TomlKind::Dot, Box::new(none_around_dot) as RuleFn),
        // space around equal sign
        (TomlKind::Equal, Box::new(space_around_eq) as RuleFn),
        // space or newline after comma in array or inline table
        (TomlKind::Comma, Box::new(space_lf_after_comma) as RuleFn),
        // space or newline after array open brace
        (
            TomlKind::OpenCurly,
            Box::new(space_lf_after_inline_table_open) as RuleFn,
        ),
        // space or newline before closing curly brace of inline table
        (
            TomlKind::CloseCurly,
            Box::new(space_lf_before_inline_table_close) as RuleFn,
        ),
        // space or newline after open brace of array
        (
            TomlKind::OpenBrace,
            Box::new(space_lf_after_array_open) as RuleFn,
        ),
        // space or newline before closing brace of array
        (
            TomlKind::CloseBrace,
            Box::new(space_lf_before_array_close) as RuleFn,
        ),
        // INDENT
        // indent after comma if siblings are indented
        (TomlKind::Comma, Box::new(indent_after_comma) as RuleFn),
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tkn_tree::parse_it;
    use std::fs::read_to_string;

    #[test]
    fn fmt_eq_space() {
        let file = "[table]\nkey=false";
        let parsed = parse_it(file).expect("parse failed").syntax();
        let fmted = Formatter::new(&parsed).format();
        assert_eq!("[table]\nkey = false\n", fmted.formatted);
    }
    #[test]
    fn fmt_eq_space_many() {
        let file = "[table]\nkey   =  false\n";
        let parsed = parse_it(file).expect("parse failed").syntax();
        let fmted = Formatter::new(&parsed).format();
        assert_eq!("[table]\nkey = false\n", fmted.formatted);
    }
    #[test]
    fn fmt_heading() {
        let file = "[table] key = false";
        let parsed = parse_it(file).expect("parse failed").syntax();
        let fmt = Formatter::new(&parsed).format();
        println!("{:#?}", fmt);
        assert_eq!(fmt.formatted, "[table]\nkey = false\n")
    }
    #[test]
    fn fmt_comma_arr() {
        let file = "key = [1,2,3]";
        let parsed = parse_it(file).expect("parse failed").syntax();
        let fmt = Formatter::new(&parsed).format();
        assert_eq!(fmt.formatted, "key = [ 1, 2, 3 ]\n")
    }
    #[test]
    fn fmt_comma_obj() {
        let file = "key={a=1,b=2}";
        let parsed = parse_it(file).expect("parse failed").syntax();
        let fmt = Formatter::new(&parsed).format();
        assert_eq!(fmt.formatted, "key = { a = 1, b = 2 }\n")
    }
    #[test]
    fn fmt_tables() {
        let file = "[table]\nkey = false [table]\nkey = 1";
        let parsed = parse_it(file).expect("parse failed").syntax();
        println!("{:#?}", parsed);
        let fmt = Formatter::new(&parsed).format();
        println!("{:#?}", fmt);
        assert_eq!(fmt.formatted, "[table]\nkey = false\n\n[table]\nkey = 1\n")
    }
    #[test]
    fn fmt_seg() {
        let file = "[table . more]\nkey = false\n";
        let parsed = parse_it(file).expect("parse failed").syntax();
        let fmt = Formatter::new(&parsed).format();
        println!("{:#?}", fmt);
        println!("{:#?}", parsed);
        assert_eq!(fmt.formatted, "[table.more]\nkey = false\n")
    }
    #[test]
    fn fmt_indent_arr() {
        let input = read_to_string("examp/indent.toml").expect("file read failed");
        let parsed = parse_it(&input).expect("").syntax();
        let fmted = Formatter::new(&parsed).format();
        println!("{:#?}", fmted);
        println!("{:#?}", parsed);
    }
    #[test]
    fn fmt_obj_comma() {
        let input = read_to_string("examp/obj_comma.toml").expect("file read failed");
        let parsed = parse_it(&input).expect("").syntax();
        let fmted = Formatter::new(&parsed).format();
        println!("{:#?}", fmted);
        println!("{:#?}", parsed);
    }
}

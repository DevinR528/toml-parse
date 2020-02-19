use std::cmp::Ordering;

use rowan::{
    Checkpoint, Children, GreenNode, GreenNodeBuilder, GreenToken, SyntaxKind, SyntaxText,
    TextRange, TextUnit, TokenAtOffset,
};

use super::tkn_tree::{
    parse_it,
    walk::{
        next_siblings, prev_non_whitespace_sibling, prev_siblings, walk_nodes, walk_non_whitespace,
        walk_tokens, walk,
    },
    SyntaxNodeExtTrait, SyntaxElement, SyntaxNode, SyntaxToken, TomlKind,
};

/// Each `Matcher` field when matched to a heading or key token
/// will be matched with `.contains()`.
pub struct Matcher<'a> {
    /// Toml headings with braces `[heading]`.
    heading: &'a [&'a str],
    /// Toml segmented heading without braces.
    segmented: &'a [&'a str],
    /// Toml heading with braces `[heading]` and the key
    /// of the array to sort.
    heading_key: &'a [(&'a str, &'a str)],
    value: TomlKind,
}

fn split_seg_last<S: AsRef<str>>(s: S) -> String {
    let open_close: &[char] = &['[', ']'];
    s.as_ref()
        .replace(open_close, "")
        .split('.')
        .last()
        .map(ToString::to_string)
        .unwrap()
}

pub fn sort_toml_items(root: &SyntaxNode, matcher: &Matcher<'_>) -> SyntaxNode {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(TomlKind::Root.into());

    for ele in sorted_tables_with_tokens(root, matcher.segmented) {
        match ele.kind() {
            TomlKind::Table => {
                // for [workspace] members = ...
                // this is heading and members is key.
                let (head, key): (Vec<_>, Vec<_>) = matcher.heading_key.iter().cloned().unzip();
                let node = ele.as_node().unwrap();
                if match_table(node, matcher.heading) {
                    add_sorted_table(node, &mut builder)
                } else if match_table(node, &head) {
                    add_table_sort_items(node, &mut builder, &key, matcher.value)
                } else {
                    add_element(ele, &mut builder)
                }
            }
            _ => add_element(ele, &mut builder),
        }
    }
    builder.finish_node();
    let green: GreenNode = builder.finish();
    SyntaxNode::new_root(green)
}

fn match_table(node: &SyntaxNode, headings: &[&str]) -> bool {
    match node.first_child().map(|n| n.kind()) {
        Some(TomlKind::Heading) => headings.iter().any(|h| node.token_text().contains(h)),
        _ => false,
    }
}

fn sorted_tables_with_tokens(
    root: &SyntaxNode,
    segmented: &[&str],
) -> impl Iterator<Item = SyntaxElement> {
    let kids = root.children_with_tokens().collect::<Vec<_>>();
    let pos = root
        .children_with_tokens()
        .enumerate()
        .filter(|(_, n)| n.as_node().map(|n| n.kind()) == Some(TomlKind::Table))
        .map(|(i, n)| {
            (
                i,
                n.as_node()
                    .unwrap()
                    .children()
                    .find(|n| n.kind() == TomlKind::Heading)
                    .map(|n| n.token_text()),
            )
        })
        .collect::<Vec<_>>();

    let mut tables = Vec::default();
    let mut start = 0;
    for (idx, key) in pos {
        let next_is_whitespace = kids
            .get(idx + 1)
            .map(|el| el.as_token().map(|t| t.kind()) == Some(TomlKind::Whitespace))
            == Some(true);

        let idx = if next_is_whitespace { idx + 1 } else { idx };

        tables.push((key, kids[start..=idx].to_vec()));
        start = idx + 1;
    }

    if start != kids.len() {
        tables.push((None, kids[start..].to_vec()))
    }

    tables.sort_by(|chunk, other| {
        let chunk_matches_heading = chunk.0.as_ref().map(|head| {
            segmented
                .iter()
                .any(|seg| head.contains(&format!("[{}", seg)))
        }) == Some(true);
        let other_matches_heading = other.0.as_ref().map(|head| {
            segmented
                .iter()
                .any(|seg| head.contains(&format!("[{}", seg)))
        }) == Some(true);

        if chunk_matches_heading && other_matches_heading {
            chunk
                .0
                .as_ref()
                .map(split_seg_last)
                .cmp(&other.0.as_ref().map(split_seg_last))
        } else {
            Ordering::Equal
        }
    });

    tables.into_iter().map(|p| p.1).flatten()
}

fn add_sorted_table(node: &SyntaxNode, builder: &mut GreenNodeBuilder) {
    builder.start_node(node.kind().into());

    if let Some(heading) = node.first_child() {
        add_node(&heading, builder);
    } else {
        unreachable!("table without heading")
    }

    let kv = node.children_with_tokens().skip(1).collect::<Vec<_>>();
    for ele in sort_key_value(&kv) {
        add_element(ele, builder);
    }

    builder.finish_node();
}

fn sort_key_value(kv: &[SyntaxElement]) -> Vec<SyntaxElement> {
    let pos = kv
        .iter()
        .enumerate()
        .filter(|(_, n)| n.as_node().map(|n| n.kind()) == Some(TomlKind::KeyValue))
        .map(|(i, n)| {
            (
                i,
                n.as_node()
                    .unwrap()
                    .children()
                    .find(|n| n.kind() == TomlKind::Key)
                    .map(|n| n.token_text()),
            )
        })
        .collect::<Vec<_>>();

    let mut keys = Vec::default();
    let mut start = 0;
    for (idx, key) in pos {
        let next_is_whitespace = kv
            .get(idx + 1)
            .map(|el| el.as_token().map(|t| t.kind()) == Some(TomlKind::Whitespace))
            == Some(true);

        let idx = if next_is_whitespace { idx + 1 } else { idx };
        keys.push((key, &kv[start..=idx]));
        start = idx + 1;
    }
    if start != kv.len() {
        keys.push((None, &kv[start..]))
    }
    keys.sort_by(|chunk, other| chunk.0.cmp(&other.0));
    keys.into_iter()
        .map(|p| p.1)
        .flatten()
        .cloned()
        .collect()
}

fn match_key(node: &SyntaxElement, keys: &[&str]) -> bool {
    match node
        .as_node()
        .map(|n| n.first_child().map(|n| n.kind()))
        .flatten()
    {
        Some(TomlKind::Key) => keys.iter().any(|h| {
            node.as_node()
                .unwrap()
                .first_child()
                .unwrap()
                .token_text()
                .contains(h)
        }),
        _ => false,
    }
}

fn add_table_sort_items(
    node: &SyntaxNode,
    builder: &mut GreenNodeBuilder,
    key: &[&str],
    node_type: TomlKind,
) {
    builder.start_node(node.kind().into());

    if let Some(heading) = node.first_child() {
        add_node(&heading, builder);
    } else {
        unreachable!("table without heading")
    }

    for ele in node.children_with_tokens().skip(1) {
        if match_key(&ele, key) {
            // this is a `KeyValue` node
            builder.start_node(ele.kind().into());
            for el in ele.as_node().unwrap().children_with_tokens() {
                match el {
                    SyntaxElement::Node(n) => match n.kind() {
                        TomlKind::Value => {
                            if n.first_child().map(|n| n.kind()) == Some(node_type) {
                                builder.start_node(TomlKind::Value.into());
                                for sorted in sort_items(n.first_child().unwrap()) {
                                    add_element(sorted, builder);
                                }
                                builder.finish_node();
                            }
                        }
                        _ => add_node(&n, builder),
                    },
                    SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
                }
            }
            builder.finish_node();
        } else {
            add_element(ele, builder);
        }
    }

    builder.finish_node();
}

fn sort_items(node: SyntaxNode) -> Vec<SyntaxElement> {
    let children = node.children_with_tokens().collect::<Vec<_>>();
    let pos = children
        .iter()
        .enumerate()
        .filter(|(_, n)| n.as_node().map(|n| n.kind()) == Some(TomlKind::ArrayItem))
        .map(|(i, n)| {
            (
                i,
                n.as_node()
                    .unwrap()
                    .children()
                    .find(|n| n.kind() == TomlKind::Value)
                    .map(|n| n.token_text()),
            )
        })
        .collect::<Vec<_>>();

    let mut sorted = Vec::default();
    let mut start = 0;
    for (idx, key) in pos {
        let next_is_whitespace = children
            .get(idx + 1)
            .map(|el| el.as_token().map(|t| t.kind()) == Some(TomlKind::Whitespace))
            == Some(true);

        let idx = if next_is_whitespace { idx + 1 } else { idx };
        sorted.push((key, &children[start..=idx]));
        start = idx + 1;
    }
    if start != children.len() {
        sorted.push((None, &children[start..]))
    }
    sorted.sort_by(|chunk, other| chunk.0.cmp(&other.0));
    sorted.into_iter()
        .map(|p| p.1)
        .flatten()
        .cloned()
        .collect()
}

fn add_node(node: &SyntaxNode, builder: &mut GreenNodeBuilder) {
    builder.start_node(node.kind().into());

    for kid in node.children_with_tokens() {
        match kid {
            SyntaxElement::Node(n) => add_node(&n, builder),
            SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
        }
    }

    builder.finish_node();
}

fn add_element(node: SyntaxElement, builder: &mut GreenNodeBuilder) {
    match node {
        SyntaxElement::Node(node) => {
            builder.start_node(node.kind().into());
            for kid in node.children_with_tokens() {
                match kid {
                    SyntaxElement::Node(n) => add_node(&n, builder),
                    SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
                }
            }
            builder.finish_node();
        }
        SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    const HEADER: Matcher<'static> = Matcher {
        heading: &["[dependencies]"],
        segmented: &["dependencies."],
        heading_key: &[("[workspace]", "members")],
        value: TomlKind::Array,
    };
    
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
        println!("{}", sorted.token_text());
        print_overlaping(&sorted, &parsed);

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
        println!("{}", sorted.token_text());
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
        println!("{:#?}", parsed);

        let sorted = sort_toml_items(&parsed, &HEADER);
        println!("{:#?}", sorted);

        assert!(!parsed.deep_eq(&sorted));
        assert_eq!(sorted.text_range(), parsed.text_range());
    }
    #[test]
    fn sort_tkns_fend() {
        let input = read_to_string("examp/fend.toml").expect("file read failed");
        let parsed = parse_it(&input).expect("parse failed").syntax();
        let parsed2 = parse_it(&input).expect("parse failed").syntax();

        assert!(parsed.deep_eq(&parsed2));
        // println!("{:#?}", parsed);
        // println!("{}", parsed.token_text());

        let sorted = sort_toml_items(&parsed, &HEADER);
        // print_overlaping(&sorted, &parsed);
        println!("{:#?}", sorted);
        // println!("{}", sorted.token_text());

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

        // idempotent
        assert!(parsed.deep_eq(&sorted));
        assert_eq!(sorted.text_range(), parsed.text_range());
    }

    #[test]
    fn sort_tkns_win() {
        let input = read_to_string("examp/win.toml").expect("file read failed");
        let parsed = parse_it(&input).expect("parse failed").syntax();
        let parsed2 = parse_it(&input).expect("parse failed").syntax();

        assert!(parsed.deep_eq(&parsed2));
        println!("{:#?}", parsed);

        let sorted = sort_toml_items(&parsed, &HEADER);
        println!("{}", sorted.token_text());

        // idempotent
        assert!(parsed.deep_eq(&sorted));
        assert_eq!(sorted.text_range(), parsed.text_range());
    }
}

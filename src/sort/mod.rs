use std::cmp::Ordering;

use rowan::{
    Checkpoint, Children, GreenNode, GreenNodeBuilder, GreenToken, SyntaxKind, SyntaxText,
    TextRange, TextUnit, TokenAtOffset,
};

use super::tkn_tree::{
    parse_it,
    walk::{
        next_siblings, prev_non_whitespace_sibling, prev_siblings, walk_nodes, walk_non_whitespace,
        walk_tokens,
    },
    Printer, SyntaxElement, SyntaxNode, SyntaxToken, TomlKind,
};

pub fn sort_toml(root: SyntaxNode, headings: &[&str]) -> SyntaxNode {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(root.kind().into());

    for ele in root.children_with_tokens() {
        match ele.kind() {
            TomlKind::Table => {
                if match_table(ele.as_node().unwrap(), headings) {
                    add_sort_table(ele.as_node().unwrap(), &mut builder)
                }
            }
            _ => match ele {
                SyntaxElement::Node(n) => add_node(n, &mut builder),
                SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
            },
        }
    }

    builder.finish_node();
    SyntaxNode::new_root(builder.finish())
}

fn match_table(node: &SyntaxNode, headings: &[&str]) -> bool {
    let node = node.first_child().unwrap();
    match node.kind() {
        TomlKind::Heading => headings.iter().any(|h| node.token_text().contains(h)),
        _ => false,
    }
}

fn add_sort_table(node: &SyntaxNode, builder: &mut GreenNodeBuilder) {
    builder.start_node(node.kind().into());

    if let Some(heading) = node.first_child() {
        add_node(heading, builder)
    } else {
        unreachable!("table without heading")
    }

    let kv = node.children_with_tokens().skip(1).collect::<Vec<_>>();
    for ele in sort_key_value(&kv) {
        add_element(ele, builder);
    }

    println!("{:#?}", kv);
    builder.finish_node();
}

fn sort_key_value(kv: &[SyntaxElement]) ->  Vec<SyntaxElement> {
    let pos = kv
        .iter()
        .enumerate()
        .filter(|(_, n)| n.as_node().map(|n| n.kind()) == Some(TomlKind::KeyValue))
        .map(|(i, n)| (i, n.as_node().unwrap().children().find(|n| n.kind() == TomlKind::Key).map(|n| n.token_text())))
        .collect::<Vec<_>>();
    
    let mut keys = Vec::default();
    let mut start = 0;
    for (idx, key) in pos {
        let idx = if kv.get(idx + 1).map(|el| el.as_token().map(|t| t.kind()) == Some(TomlKind::Whitespace)) == Some(true) {
            idx + 1
        } else {
            idx
        };
        println!("{:?}", &kv[start..=idx]);
        keys.push((key, &kv[start..=idx]));
        start = idx + 1;
    }

    keys.sort_by(|chunk, other| {
        chunk.0.cmp(&other.0)
    });
    keys.into_iter().map(|p| p.1).flatten().cloned().collect::<Vec<_>>()
}

fn add_node(node: SyntaxNode, builder: &mut GreenNodeBuilder) {
    builder.start_node(node.kind().into());

    for kid in node.children_with_tokens() {
        match kid {
            SyntaxElement::Node(n) => add_node(n, builder),
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
                    SyntaxElement::Node(n) => add_node(n, builder),
                    SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
                }
            }
            builder.finish_node();
        },
        SyntaxElement::Token(t) => {
            builder.token(t.kind().into(), t.text().clone())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn comment_tkns() {
        let file = r#"# comment
[deps]
number = 1234
# comment
alpha = "beta"
"#;
        let parsed = parse_it(file).expect("parse failed").syntax();
        println!("{:#?}", parsed);
        let sorted = sort_toml(parsed, &["deps"]);
        println!("{:#?}", sorted)
    }

    #[test]
    fn sort_tkns() {
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let parsed = parse_it(&input).expect("parse failed").syntax();
        println!("{:#?}", parsed);
        let sorted = sort_toml(parsed, &["dependencies"]);
        println!("{:#?}", sorted)
    }
}

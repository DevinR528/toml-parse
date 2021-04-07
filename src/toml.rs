use toml_edit::{Array, Document, Item, Iter, Table, Value};

use crate::Matcher;

enum Heading {
    Start(String),
}

fn sort_inner(table: &mut Table, keys: &mut Vec<String>) {
    for (head, item) in table.iter_mut() {
        match item {
            Item::Value(_) => break,
            Item::Table(table) => {
                if head == "workspace" {
                    panic!("{:?}", table);
                }
                keys.push(head.to_string());
                sort_inner(table, keys)
            }
            Item::ArrayOfTables(arr) => todo!("ArrayOfTables: {:?}", arr),
            Item::None => break,
        }
    }

    table.sort_values();
}

fn sort_arr(arr: &mut Array) {
    let mut sortable = vec![];
    for idx in 0..arr.len() {
        sortable.push(arr.remove(idx));
    }
    sortable.sort_unstable_by_key(|a| a.to_string());
    for item in sortable {
        // Can't really fail here since we are using a previously valid `Array`
        arr.push_formatted(item).unwrap();
    }
}

pub fn is_sort_toml(input: &str, matcher: Matcher) -> Document {
    let mut toml = input.parse::<Document>().unwrap();

    // This takes care of `[workspace] members = [...]`
    for (heading, key) in matcher.heading_key {
        if let Item::Table(table) = &mut toml[&heading] {
            if let Item::Value(Value::Array(arr)) = &mut table[key] {
                sort_arr(arr)
            }
        }
    }

    let mut headings = vec![];
    for (head, item) in toml.as_table_mut().iter_mut() {
        if !matcher.heading.contains(&head) {
            continue;
        }
        match item {
            Item::Table(table) => {
                headings.push(head.to_string());
                sort_inner(table, &mut headings)
            }
            Item::None => continue,
            _ => unreachable!("Top level toml must be tables"),
        }
    }
    panic!("{:?}", headings);

    toml
}

#[cfg(test)]
mod test {
    use super::Matcher;
    use std::fs::read_to_string;

    const HEADERS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    const HEADER_SEG: [&str; 3] = ["dependencies.", "dev-dependencies.", "build-dependencies."];

    const MATCHER: Matcher<'_> = Matcher {
        heading: &HEADERS,
        segmented: &HEADER_SEG,
        heading_key: &[("workspace", "members"), ("workspace", "exclude")],
    };
    #[test]
    fn hello() {}

    #[test]
    fn toml_edit_check() {
        let input = read_to_string("examp/seg_sort.toml").expect("file read failed");
        let parsed = super::is_sort_toml(&input, MATCHER);
        println!("{}", parsed.to_string())
    }
}

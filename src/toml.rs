use toml_edit::{Array, Document, Item, Table, Value};

use crate::Matcher;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Heading {
    Start(String),
    Next(Vec<String>),
    Complete(Vec<String>),
}

fn sort_inner(table: &mut Table, keys: &mut Vec<Heading>) {
    for (head, item) in table.iter_mut() {
        match item {
            Item::Value(_) => {
                let next = match keys.pop().unwrap() {
                    Heading::Start(s) => Heading::Complete(vec![s]),
                    Heading::Next(segs) => Heading::Complete(segs),
                    // This should not be possible
                    complete => complete,
                };
                keys.push(next);
                continue;
            }
            Item::Table(table) => {
                let next = match keys.pop().unwrap() {
                    Heading::Start(s) => Heading::Next(vec![s, head.to_string()]),
                    Heading::Next(mut segs) => {
                        segs.push(head.to_string());
                        Heading::Next(segs)
                    }
                    // This should not be possible
                    Heading::Complete(segs) => {
                        let next = vec![segs[0].clone(), head.into()];
                        keys.push(Heading::Complete(segs));
                        Heading::Next(next)
                    }
                };
                keys.push(next);
                sort_inner(table, keys)
            }
            Item::ArrayOfTables(arr) => todo!("ArrayOfTables: {:?}", arr),
            Item::None => break,
        }
    }

    table.sort_values();
}

fn sort_arr(arr: &mut Array) {
    println!("{:?}", arr);
    let mut sortable = vec![];
    for _ in 0..arr.iter().count() {
        sortable.push(arr.remove(0));
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
        // Since this `&mut toml[&heading]` is like `SomeMap.entry(key).or_insert(Item::None)`
        // we only want to do it if we know the heading is there already
        if toml.as_table().contains_key(heading) {
            if let Item::Table(table) = &mut toml[heading] {
                if table.contains_key(key) {
                    if let Item::Value(Value::Array(arr)) = &mut table[key] {
                        sort_arr(arr)
                    }
                }
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
                headings.push(Heading::Start(head.to_string()));
                sort_inner(table, &mut headings);
            }
            Item::None => continue,
            _ => unreachable!("Top level toml must be tables"),
        }
    }

    headings.sort();

    for (idx, heading) in headings.into_iter().enumerate() {
        if let Heading::Complete(segs) = heading {
            let mut table = &mut Item::None;

            for seg in &segs {
                table = if let Item::None = table {
                    &mut toml[seg]
                } else {
                    &mut table[seg]
                };
            }
            match &mut table {
                Item::Table(tab) => {
                    // tab.set_position(idx);
                    // count += 1;
                }
                Item::ArrayOfTables(arr) => todo!("{:?}", arr),
                Item::Value(_) => unreachable!("Nope"),
                Item::None => continue,
            }
        }
    }

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
        let input = read_to_string("examp/ftop.toml").expect("file read failed");
        let parsed = super::is_sort_toml(&input, MATCHER);
        println!("{}", parsed.to_string_in_original_order())
    }
}

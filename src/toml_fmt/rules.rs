use rowan::Direction;

use super::block::Block;
use super::tkn_tree::TomlKind;
use super::ws::{calc_indent, Space, SpaceLoc, SpaceValue, WhiteSpace};

const LF_BEFORE: Space = Space {
    value: SpaceValue::Newline,
    loc: SpaceLoc::Before,
};

const MULTI_LF_BEFORE: Space = Space {
    value: SpaceValue::MultiLF(2),
    loc: SpaceLoc::Before,
};


const MAYBE_LF_BEFORE: Space = Space {
    value: SpaceValue::SingleOptionalNewline,
    loc: SpaceLoc::Before,
};

const SPACE_BEFORE: Space = Space {
    value: SpaceValue::Single,
    loc: SpaceLoc::Before,
};

const NONE: Space = Space {
    value: SpaceValue::None,
    loc: SpaceLoc::After,
};

pub(crate) fn lf_after_heading(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if l_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::Heading)
        && !r_blk.whitespace().match_space_before(LF_BEFORE)
        && l_blk.kind() == TomlKind::CloseBrace
        && r_blk.token().parent().kind() != TomlKind::ArrayHeading
    {
        return Some(WhiteSpace::from_rule(&LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn lf_after_table(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    let not_first_table = if let Some(table) = r_blk
        .token()
        .ancestors()
        .find(|node| node.kind() == TomlKind::Table)
    {
        table.prev_sibling().is_some()
    } else {
        false
    };
    let not_comment = l_blk.kind() != TomlKind::CommentText;
    let once_for_array_table = if r_blk.token().parent().kind() == TomlKind::ArrayHeading {
        r_blk.token().next_sibling_or_token().unwrap().kind() == TomlKind::OpenBrace
    } else {
        true
    };

    if r_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::Heading)
        && !r_blk.whitespace().match_space_before(MULTI_LF_BEFORE)
        && r_blk.kind() == TomlKind::OpenBrace
        && not_first_table
        && not_comment
        && once_for_array_table
    {
        return Some(WhiteSpace::from_rule(&MULTI_LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn space_lf_after_comma(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if !r_blk.whitespace().match_space_before(MAYBE_LF_BEFORE) && l_blk.kind() == TomlKind::Comma {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&MAYBE_LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn space_lf_after_inline_table_open(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if !r_blk.whitespace().match_space_before(MAYBE_LF_BEFORE)
        && l_blk.kind() == TomlKind::OpenCurly
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&MAYBE_LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn space_lf_before_inline_table_close(
    l_blk: &Block,
    r_blk: &Block,
) -> Option<WhiteSpace> {
    if !r_blk.whitespace().match_space_before(MAYBE_LF_BEFORE)
        && r_blk.kind() == TomlKind::CloseCurly
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&MAYBE_LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn space_lf_after_array_open(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if l_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::Value)
        && !r_blk.whitespace().match_space_before(MAYBE_LF_BEFORE)
        && l_blk.kind() == TomlKind::OpenBrace
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&MAYBE_LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn space_lf_before_array_close(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if r_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::Value)
        && !r_blk.whitespace().match_space_before(MAYBE_LF_BEFORE)
        && r_blk.kind() == TomlKind::CloseBrace
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&MAYBE_LF_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn space_around_eq(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if r_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::KeyValue)
        && !r_blk.whitespace().match_space_before(SPACE_BEFORE)
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&SPACE_BEFORE, l_blk, r_blk));
    }
    None
}

pub(crate) fn none_around_dot(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    if r_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::SegIdent)
        && !r_blk.whitespace().match_space_before(NONE)
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
        return Some(WhiteSpace::from_rule(&NONE, l_blk, r_blk));
    }
    None
}

#[allow(clippy::collapsible_if)]
pub(crate) fn indent_after_comma(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    let (has_indent, is_tab, (level, alignment)) = if let Some(arr_item) = r_blk
        .token()
        .ancestors()
        .find(|n| n.kind() == TomlKind::ArrayItem)
    {
        if let Some(ws) = arr_item.siblings_with_tokens(Direction::Prev).find(|el| {
            el.as_token().map(|t| {
                // println!("TKN {:?}", t);
                match t.text().as_bytes() {
                    // `\n\s...`
                    [10, 32, ..] => true,
                    // `\n\t`
                    [10, 9, ..] => true,
                    [] | [_] | [_, _, ..] => false,
                }
            }) == Some(true)
        }) {
            let raw_ws = ws.as_token().unwrap().text();
            (true, raw_ws.contains("\t"), calc_indent(raw_ws))
        } else if let Some(ws) = arr_item.siblings_with_tokens(Direction::Next).find(|el| {
            // println!("EL NEXT {:#?}", el);
            el.as_token().map(|t| {
                match t.text().as_bytes() {
                    // `\n\s...`
                    [10, 32, ..] => true,
                    // `\n\t`
                    [10, 9, ..] => true,
                    [] | [_] | [_, _, ..] => false,
                }
            }) == Some(true)
        }) {
            let raw_ws = ws.as_token().unwrap().text();
            (true, raw_ws.contains("\t"), calc_indent(raw_ws))
        } else {
            (false, false, (0, 0))
        }
    } else {
        (false, false, (0, 0))
    };
    // println!("{} {} {}", has_indent, level, alignment);
    if l_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::ArrayItem)
    {
        if r_blk.kind() == TomlKind::CloseBrace {
            let eol = Space {
                value: SpaceValue::Newline,
                loc: SpaceLoc::Before,
            };
            return Some(WhiteSpace::from_rule(&eol, l_blk, r_blk));
        }

        let indent = Space {
            value: SpaceValue::Indent { level, alignment, is_tab, },
            loc: SpaceLoc::Before,
        };
        // println!(
        //     "FIRST {:#?} {:#?} {}",
        //     l_blk,
        //     r_blk,
        //     r_blk.whitespace().space_before == indent
        // );
        if l_blk.kind() == TomlKind::Comma
            && has_indent
            && r_blk.whitespace().space_before != indent
        {
            return Some(WhiteSpace::from_rule(&indent, l_blk, r_blk));
        }
    }
    None
}

#[allow(clippy::collapsible_if)]
pub(crate) fn indent_after_open_brace(l_blk: &Block, r_blk: &Block) -> Option<WhiteSpace> {
    let (has_indent, is_tab, (level, alignment)) = if let Some(arr_item) = l_blk
        .token()
        .next_sibling_or_token()
        .and_then(|t| {
            if t.kind() == TomlKind::ArrayItem {
                t.as_node().cloned()
            } else {
                None
            }
        })
    {
        // println!("{:#?} {:#?}", l_blk, r_blk);
        if let Some(ws) = arr_item.siblings_with_tokens(Direction::Next).find(|el| {
                // println!("el {:#?}", el);
                el.as_token().map(|t| {
                    match t.text().as_bytes() {
                        // `\n\s...`
                        [10, 32, ..] => {
                            // println!("prev token {:#?}", t.prev_sibling_or_token());
                            t.prev_sibling_or_token()
                                .map(|t| t.kind() == TomlKind::ArrayItem)
                                == Some(true)
                        }
                        // `\n\t`
                        [10, 9, ..] => {
                            t.prev_sibling_or_token()
                                .map(|t| t.kind() == TomlKind::ArrayItem)
                                == Some(true)
                        }
                        [] | [_] | [_, _, ..] => false,
                    }
                }) == Some(true)
            }) {
                let raw_ws = ws.as_token().unwrap().text();
                (true, raw_ws.contains("\t"), calc_indent(raw_ws))
            } else {
                (false, false, (0, 0))
            }
        } else {
            (false, false, (0, 0))
        };
        // println!("{} {} {}", has_indent, level, alignment);
        let indent = Space {
            value: SpaceValue::Indent { level, alignment, is_tab, },
            loc: SpaceLoc::Before,
        };
        if l_blk.kind() == TomlKind::OpenBrace
            && has_indent
            && r_blk.whitespace().space_before != indent
        {
            return Some(WhiteSpace::from_rule(&indent, l_blk, r_blk));
        }
    None
}

#![allow(unused)]
#![allow(dead_code)]

use rowan::Direction;

use super::block::Block;
use super::tkn_tree::TomlKind;
use super::ws::{calc_indent, Space, SpaceLoc, SpaceValue, WhiteSpace};

const LF_AFTER: Space = Space {
    value: SpaceValue::Newline,
    loc: SpaceLoc::After,
};

const LF_BEFORE: Space = Space {
    value: SpaceValue::Newline,
    loc: SpaceLoc::Before,
};

const MULTI_LF_BEFORE: Space = Space {
    value: SpaceValue::MultiLF(2),
    loc: SpaceLoc::Before,
};

const MAYBE_LF_AFTER: Space = Space {
    value: SpaceValue::SingleOptionalNewline,
    loc: SpaceLoc::After,
};

const MAYBE_LF_BEFORE: Space = Space {
    value: SpaceValue::SingleOptionalNewline,
    loc: SpaceLoc::Before,
};

const SPACE_BEFORE: Space = Space {
    value: SpaceValue::Single,
    loc: SpaceLoc::Before,
};

const SPACE_AFTER: Space = Space {
    value: SpaceValue::Single,
    loc: SpaceLoc::After,
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
    {
        // println!("MATCH {:#?} {:#?}", l_blk, r_blk);
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

    if r_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::Heading)
        && !r_blk.whitespace().match_space_before(MULTI_LF_BEFORE)
        && r_blk.kind() == TomlKind::OpenBrace
        && not_first_table
        && not_comment
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
    let (has_indent, (level, alignment)) = if let Some(table) = r_blk
        .token()
        .ancestors()
        .find(|n| n.kind() == TomlKind::ArrayItem)
    {
        if let Some(ws) = table.siblings_with_tokens(Direction::Prev).find(|el| {
            el.as_token().map(|t| {
                t.text().contains("\n ")
                    && t.prev_sibling_or_token()
                        .map(|t| t.as_token().map(|t| t.kind() == TomlKind::OpenBrace))
                        .flatten()
                        == Some(true)
            }) == Some(true)
        }) {
            (
                true,
                calc_indent(ws.as_token().unwrap().text().matches(' ').count() as u32),
            )
        } else if let Some(ws) = table.siblings_with_tokens(Direction::Next).find(|el| {
            el.as_token().map(|t| {
                t.text().contains("\n ")
                    && t.prev_sibling_or_token()
                        .map(|t| t.as_token().map(|t| t.kind() == TomlKind::OpenBrace))
                        .flatten()
                        == Some(true)
            }) == Some(true)
        }) {
            (
                true,
                calc_indent(ws.as_token().unwrap().text().matches(' ').count() as u32),
            )
        } else {
            (false, (0, 0))
        }
    } else {
        (false, (0, 0))
    };
    // println!("{} {} {}", has_indent, level, alignment);
    if l_blk
        .token()
        .ancestors()
        .any(|n| n.kind() == TomlKind::ArrayItem)
    {
        let indent = Space {
            value: SpaceValue::Indent { level, alignment },
            loc: SpaceLoc::Before,
        };
        // println!(
        //     "FIRST {:#?} {:#?} {}",
        //     l_blk,
        //     r_blk,
        //     r_blk.whitespace().space_before == indent
        // );
        if r_blk.kind() == TomlKind::CloseBrace {
            let eol = Space {
                value: SpaceValue::Newline,
                loc: SpaceLoc::Before,
            };
            return Some(WhiteSpace::from_rule(&eol, l_blk, r_blk));
        }

        if l_blk.kind() == TomlKind::Comma
            && has_indent
            && r_blk.whitespace().space_before != indent
        {
            return Some(WhiteSpace::from_rule(&indent, l_blk, r_blk));
        }
    }
    None
}

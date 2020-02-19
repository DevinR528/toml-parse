use std::fmt;

use super::tkn_tree::{
    walk::{
        next_siblings, prev_non_whitespace_sibling, prev_siblings, walk_nodes, walk_non_whitespace,
        walk_tokens, walk,
    },
    SyntaxNodeExtTrait, SyntaxElement, SyntaxNode, SyntaxToken, TomlKind::{self, *},
};
use super::Block;

pub(crate) const USER_INDENT_SIZE: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpaceValue {
    /// Single whitespace char, like `' '`
    Single,
    /// Single whitespace char, like `' '`, but preserve existing line break.
    SingleOptionalNewline,
    /// A single newline (`\n`) char
    Newline,
    /// No whitespace at all.
    None,
    /// No space, but preserve existing line break.
    NoneOptionalNewline,
    /// If the parent element fits into a single line, a single space.
    /// Otherwise, at least one newline.
    /// Existing newlines are preserved.
    SingleOrNewline,
    /// If the parent element fits into a single line, no space.
    /// Otherwise, at least one newline.
    /// Existing newlines are preserved.
    NoneOrNewline,
    /// Number of spaces this is only for `Whitespace` held by `Block`.
    MultiSpace(u32),
    /// Number of new lines this is only for `Whitespace` held by `Block`.
    MultiLF(u32),
    /// Number of spaces that indent is made of `'\n\s\s\s\s'`.
    Indent { level: u32, alignment: u32},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpaceLoc {
    /// Before the element.
    Before,
    /// After the element.
    After,
    /// On the both sides of the element.
    Around,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Space {
    /// How much space to add.
    pub(crate) value: SpaceValue,
    /// Should the space be added before, after or around the element?
    pub(crate) loc: SpaceLoc,
}

impl Space {
    fn empty_before() -> Space {
        Self { loc: SpaceLoc::Before, value: SpaceValue::None }
    }
    fn empty_after() -> Space {
        Self { loc: SpaceLoc::After, value: SpaceValue::None }
    }
    fn before(token: SyntaxToken) -> Space {
        if !is_ws(&token) {
            return Self::empty_before();
        }
        let value = calc_space_value(&token);
        Self { loc: SpaceLoc::Before, value }
    }
    fn after(token: SyntaxToken) -> Space {
        if !is_ws(&token) {
            return Self::empty_after();
        }
        let value = calc_space_value(&token);
        Self { loc: SpaceLoc::After, value }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            SpaceValue::Single => write!(f, " "),
            SpaceValue::Newline => writeln!(f),
            SpaceValue::Indent{ level, alignment, } => write!(f, "\n{}", " ".repeat((level * USER_INDENT_SIZE + alignment) as usize)),
            SpaceValue::None => write!(f, ""),
            _ => {
                // unreachable!("no other writable variants")
                write!(f, " {:?} ", self.value)
            }
        }
    }
}

pub struct WhiteSpace {
    pub(crate) space_before: Space,
    pub(crate) space_after: Space,
}

impl WhiteSpace {
    pub(crate) fn new(token: &SyntaxToken) -> WhiteSpace {
        let (space_before, space_after) = match (token.prev_token(), token.next_token()) {
            (Some(pre), Some(post)) => (Space::before(pre), Space::after(post)),
            (Some(pre), _) => (Space::before(pre), Space::empty_after()),
            (_, Some(post)) => (Space::empty_before(), Space::after(post)),
            (_, _) => unreachable!("next or previous token returned a node"),
        };

        Self { space_before, space_after }
    }
}
fn is_ws(token: &SyntaxToken) -> bool {
    token.kind() == TomlKind::Whitespace
}

fn calc_indent(len: u32) -> (u32, u32) {
    let level = len / USER_INDENT_SIZE;
    let alignment = len % USER_INDENT_SIZE;
    (level, alignment)
}

fn calc_space_value(tkn: &SyntaxToken) -> SpaceValue {
    let orig = tkn.text().as_str();
    let tkn_len = orig.chars().count();
    // indent is `\n\s\s\s\s` or some variation
    if orig.contains('\n') && orig.contains(' ') {
        let (level, alignment) = calc_indent(orig.matches(' ').count() as u32);
        SpaceValue::Indent { level, alignment }
    // just new line
    } else if orig.contains('\n') {
        if tkn_len == 1 {
            SpaceValue::Newline
        } else {
            SpaceValue::MultiLF((orig.matches('\n').count()) as u32)
        }
    // just spaces
    } else if orig.contains(' ') {
        if tkn_len == 1 {
            SpaceValue::Single
        } else {
            SpaceValue::MultiSpace((orig.matches(' ').count()) as u32)
        }
    } else {
        SpaceValue::None
    }
}

fn filter_nodes(pre: Option<SyntaxElement>, post: Option<SyntaxElement>) -> (Space, Space) {
    match (pre, post) {
        (Some(SyntaxElement::Token(pre)), Some(SyntaxElement::Token(post))) => {
            (Space::before(pre), Space::after(post))
        }
        (Some(SyntaxElement::Token(pre)), _) => (Space::before(pre), Space::empty_after()),
        (_, Some(SyntaxElement::Token(post))) => (Space::empty_before(), Space::after(post)),
        (None, None) => (Space::empty_before(), Space::empty_after()),
        _non_token_tuple => {
            // println!("this is anything that is not a token {:?}", a);
            (Space::empty_before(), Space::empty_after())
        }
    }
}

// /// TODO some left block parents may have diff contains, check when SpaceLoc::After
// fn process_space_value(blk: &Block, space: &SpacingRule) -> SpaceValue {
//     use SpaceValue::*;
//     match space.space.value {
//         Newline | MultiLF(_) => Newline,
//         Single | MultiSpace(_) => Single,
//         NoneOptionalNewline | NoneOrNewline => {
//             if blk.siblings_contain("\n") {
//                 Newline
//             } else {
//                 SpaceValue::None
//             }
//         }
//         SingleOptionalNewline | SingleOrNewline => {
//             if blk.siblings_contain("\n") {
//                 Newline
//             } else {
//                 Single
//             }
//         }
//         Indent {level, alignment, } => Indent { level, alignment, },
//         None => space.space.value,
//     }
// }

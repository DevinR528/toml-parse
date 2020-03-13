use std::fmt;

use super::tkn_tree::{SyntaxToken, TomlKind};
use super::Block;

pub(crate) const USER_INDENT_SIZE: u32 = 4;

#[allow(dead_code)]
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
    Indent { level: u32, alignment: u32, is_tab: bool },
}

#[allow(dead_code)]
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
        Self {
            loc: SpaceLoc::Before,
            value: SpaceValue::None,
        }
    }
    fn empty_after() -> Space {
        Self {
            loc: SpaceLoc::After,
            value: SpaceValue::None,
        }
    }
    fn before(token: SyntaxToken) -> Space {
        if !is_ws(&token) {
            return Self::empty_before();
        }
        let value = calc_space_value(&token);
        Self {
            loc: SpaceLoc::Before,
            value,
        }
    }
    fn after(token: SyntaxToken) -> Space {
        if !is_ws(&token) {
            return Self::empty_after();
        }
        let value = calc_space_value(&token);
        Self {
            loc: SpaceLoc::After,
            value,
        }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            SpaceValue::Single => write!(f, " "),
            SpaceValue::Newline => writeln!(f),
            SpaceValue::Indent { level, alignment, is_tab: false } => write!(
                f,
                "\n{}",
                " ".repeat((level * USER_INDENT_SIZE + alignment) as usize)
            ),
            SpaceValue::Indent { level, is_tab: true, .. } => write!(
                f,
                "\n{}",
                "\t".repeat(level as usize)
            ),
            SpaceValue::MultiLF(count) => write!(f, "{}", "\n".repeat(count as usize)),
            SpaceValue::MultiSpace(count) => write!(f, "{}", " ".repeat(count as usize)),
            SpaceValue::None => write!(f, ""),
            _ => {
                // unreachable!("no other writable variants")
                write!(f, " {:?} ", self.value)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WhiteSpace {
    pub(crate) space_before: Space,
    pub(crate) space_after: Space,
}

impl AsRef<WhiteSpace> for WhiteSpace {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl WhiteSpace {
    pub(crate) fn new(token: &SyntaxToken) -> WhiteSpace {
        let (space_before, space_after) = match (token.prev_token(), token.next_token()) {
            (Some(pre), Some(post)) => (Space::before(pre), Space::after(post)),
            (Some(pre), _) => (Space::before(pre), Space::empty_after()),
            (_, Some(post)) => (Space::empty_before(), Space::after(post)),
            (_, _) => unreachable!("next or previous token returned a node"),
        };

        Self {
            space_before,
            space_after,
        }
    }

    pub(crate) fn from_rule(space: &Space, l_blk: &Block, r_blk: &Block) -> WhiteSpace {
        match space.loc {
            SpaceLoc::Before => {
                let space_before = Space {
                    loc: space.loc,
                    value: process_space_value(r_blk, space),
                };

                Self {
                    space_before,
                    space_after: Space::empty_after(),
                }
            }
            SpaceLoc::After => {
                let space_after = Space {
                    loc: SpaceLoc::Before,
                    value: process_space_value(l_blk, space),
                };
                Self {
                    space_before: space_after,
                    space_after: Space::empty_after(),
                }
            }
            SpaceLoc::Around => {
                let space_before = Space {
                    loc: SpaceLoc::Before,
                    value: process_space_value(r_blk, space),
                };
                Self {
                    space_before,
                    space_after: r_blk.whitespace().space_after,
                }
            }
        }
    }

    /// Returns true if `space` a `SpaceRule` matches `Whitespace` value.
    pub(crate) fn match_space_before(&self, space: Space) -> bool {
        use SpaceValue::*;
        if self.space_before.value == space.value {
            return true;
        }
        match space.value {
            Single => match self.space_before.value {
                Single => true,
                _ => false,
            },
            SingleOrNewline => match self.space_before.value {
                Single | Newline | Indent { .. } => true,
                _ => false,
            },
            SingleOptionalNewline => match self.space_before.value {
                Single | Newline | Indent { .. } => true,
                _ => false,
            },
            // TODO make sure valid
            Newline => match self.space_before.value {
                Newline | Indent { .. } => true,
                _ => false,
            },
            NoneOrNewline => match self.space_before.value {
                Newline | Indent { .. } => true,
                _ => false,
            },
            NoneOptionalNewline => match self.space_before.value {
                Newline | Indent { .. } => true,
                _ => false,
            },
            // TODO from here on the rules never set these they will
            // never be checked.
            MultiSpace(len) => match self.space_before.value {
                Single => len == 1,
                MultiSpace(num) => len == num,
                _ => false,
            },
            MultiLF(len) => match self.space_before.value {
                Newline | Indent { .. } => len == 1,
                MultiLF(num) => len == num,
                _ => false,
            },
            Indent { .. } => match self.space_before.value {
                Newline | Indent { .. } => true,
                MultiLF(len) => len == 1,
                _ => false,
            },
            None => None == self.space_before.value,
        }
    }
}

fn is_ws(token: &SyntaxToken) -> bool {
    token.kind() == TomlKind::Whitespace
}

pub(crate) fn calc_indent(ws: &str) -> (u32, u32) {
    let len = if ws.contains("\n ") {
        ws.matches(" ").count() as u32
    } else if ws.contains("\t") {
        ws.matches("\t").count() as u32 * USER_INDENT_SIZE
    } else {
        0
    };
    let level = len / USER_INDENT_SIZE;
    let alignment = len % USER_INDENT_SIZE;
    (level, alignment)
}

fn calc_space_value(tkn: &SyntaxToken) -> SpaceValue {
    let orig = tkn.text().as_str();
    let tkn_len = orig.chars().count();
    // indent is `\n\s\s\s\s` or some variation
    if orig.contains('\n') && (orig.contains(' ') || orig.contains('\t')) {
        let (level, alignment) = calc_indent(orig);
        SpaceValue::Indent { level, alignment, is_tab: orig.contains('\t') }
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

/// TODO some left block parents may have diff contains, check when SpaceLoc::After
fn process_space_value(blk: &Block, space: &Space) -> SpaceValue {
    use SpaceValue::*;
    match space.value {
        Newline => Newline,
        MultiLF(len) => MultiLF(len),
        Single => Single,
        MultiSpace(len) => MultiSpace(len),
        NoneOptionalNewline | NoneOrNewline => {
            if blk.parents_contain("\n") {
                Newline
            } else {
                SpaceValue::None
            }
        }
        SingleOptionalNewline | SingleOrNewline => {
            if blk.parents_contain("\n") {
                Newline
            } else {
                Single
            }
        }
        Indent { level, alignment, is_tab } => Indent { level, alignment, is_tab },
        None => space.value,
    }
}

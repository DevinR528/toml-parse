use std::ops::Range;

pub(crate) mod err;

type TextRange = Range<usize>;

/// TODO fix pass by ref
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn cmp_tokens(ch: &char, chars: &[char]) -> bool {
    chars.iter().any(|c| c == ch)
}

pub(crate) const EOL: &[char] = &['\n', '\r'];
pub(crate) const WHITESPACE: &[char] = &[' ', '\n', '\t', '\r'];

pub(crate) const QUOTE: &[char] = &['\"', '\''];
pub(crate) const ARRAY_ITEMS: &[char] = &[',', ']'];
pub(crate) const INLINE_ITEMS: &[char] = &[',', '}'];

pub(crate) const NUM_END: &[char] = &['\n', '\r', ',', ']', ' ', '}'];
pub(crate) const INT_END: &[char] = &['\n', '\r', ',', '.', ']', ' ', '}'];
pub(crate) const BOOL_END: &[char] = &['\n', '\r', ',', ']', ' ', '}'];
pub(crate) const KEY_END: &[char] = &[' ', ',', '='];
pub(crate) const IDENT_END: &[char] = &[' ', '\n', '\t', '\r', '='];
pub(crate) const SEG_END: &[char] = &[' ', '.', ']', '"'];

pub(crate) const DATE_END: &[char] = &['\n', '\r', ',', ']', '}'];
pub(crate) const DATE_LIKE: &[char] = &['-', '/', ':', 'T'];
pub(crate) const DATE_TIME: &[char] = &[' ', 'T'];
pub(crate) const DATE_CHAR: &[char] = &['-'];
pub(crate) const TIME_CHAR: &[char] = &[':', '+'];

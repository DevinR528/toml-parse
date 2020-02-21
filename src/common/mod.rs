pub(crate) mod err;

/// TODO fix pass by ref
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn cmp_tokens(ch: &char, chars: &[char]) -> bool {
    chars.iter().any(|c| c == ch)
}

pub trait GroupBy<T> {
    type Item;
    fn group_by<P>(self, predicate: P) -> (Vec<Self::Item>, Vec<Self::Item>)
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool;
}

impl<I: IntoIterator<Item = T>, T> GroupBy<T> for I {
    type Item = T;
    fn group_by<P>(self, mut predicate: P) -> (Vec<Self::Item>, Vec<Self::Item>)
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let mut t = Vec::default();
        let mut f = Vec::default();
        for x in self.into_iter() {
            if predicate(&x) {
                t.push(x)
            } else {
                f.push(x)
            }
        }
        (t, f)
    }
}

pub(crate) const EOL: &[char] = &['\n', '\r'];
pub(crate) const WHITESPACE: &[char] = &[' ', '\n', '\t', '\r'];

#[allow(unused)]
pub(crate) const QUOTE: &[char] = &['\"', '\''];
#[allow(unused)]
pub(crate) const ARRAY_ITEMS: &[char] = &[',', ']'];
#[allow(unused)]
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

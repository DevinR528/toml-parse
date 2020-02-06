use std::cell::Cell;
use std::ops::Range;

use super::err::{ParseTomlError, TomlErrorKind, TomlResult};

type TextRange = Range<usize>;

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

/// TODO fix pass by ref
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn cmp_tokens(ch: &char, chars: &[char]) -> bool {
    chars.iter().any(|c| c == ch)
}

#[derive(Debug, Clone)]
pub struct Stack<'a> {
    input: &'a str,
    stack: Vec<char>,
    pos: (usize, usize),
}

impl<'s> Stack<'s> {
    pub fn new(input: &'s str, pos: (usize, usize)) -> Stack<'s> {
        Self {
            input,
            stack: Vec::default(),
            pos,
        }
    }

    pub fn eat(&mut self, input: char) -> TomlResult<()> {
        match input {
            '{' | '[' | '(' => self.push(input),
            '}' | ']' | ')' => self.pop(),
            _ => Ok(()),
        }
    }

    pub fn push(&mut self, ch: char) -> TomlResult<()> {
        self.stack.push(ch);
        Ok(())
    }
    pub fn pop(&mut self) -> TomlResult<()> {
        if self.stack.get(0).is_some() {
            self.stack.remove(0);
            Ok(())
        } else {
            let msg = "bracket mismatch";
            let tkn = self
                .stack
                .get(0)
                .map(|c| format!("{}", c)).unwrap_or_default();
            Err(ParseTomlError::new(
                msg.into(),
                TomlErrorKind::UnexpectedToken {
                    tkn,
                    ln: self.pos.0,
                    col: self.pos.1,
                },
            ))
        }
    }
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Fork<'a> {
    input: &'a [char],
    peek: Cell<usize>,
}

impl<'f> Fork<'f> {
    pub fn reset_peek(&self) {
        self.peek.set(0);
    }

    pub fn adv_peek(&self) -> usize {
        self.peek.get() + 1
    }

    pub fn peek(&self) -> Option<&char> {
        self.input.get(self.adv_peek())
    }

    pub fn seek(&self, count: usize) -> Option<String> {
        let start = self.peek.get();
        let end = start + count;
        if end >= self.input.len() {
            return None;
        }
        Some(self.input[start..end].iter().collect())
    }
}

#[derive(Debug, Clone)]
pub struct Muncher<'a> {
    text: &'a str,
    input: Vec<char>,
    peek: Cell<usize>,
    next: usize,
}

impl<'a> Muncher<'a> {
    /// Creates a new `Muncher` of the given input.
    /// 
    /// # Example
    /// ```
    /// use toml_parse::Muncher;
    /// 
    /// let input = "parsable input";
    /// let munch = Muncher::new(input);
    /// ```
    pub fn new(input: &'a str) -> Self {
        Self {
            text: input,
            input: input.chars().collect(),
            peek: Cell::new(0),
            next: 0,
        }
    }

    /// A peekable fork that does not alter the position of
    /// the `Muncher`
    /// 
    /// # Example
    /// ```
    /// use toml_parse::Muncher;
    /// 
    /// let input = "abcde";
    /// let mut munch = Muncher::new(input);
    /// assert_eq!(munch.eat(), Some('a'));
    /// 
    /// let fork = munch.fork();
    /// assert_eq!(fork.peek(), Some(&'b'));
    /// 
    /// assert_eq!(munch.eat(), Some('b'));
    /// assert_eq!(munch.eat(), Some('c'));
    /// 
    /// ``` 
    pub fn fork(&self) -> Fork {
        Fork {
            input: &self.input[self.next - 1..],
            peek: Cell::new(0),
        }
    }


    pub fn text(&self) -> &str {
        self.text
    }

    pub fn position(&self) -> usize {
        self.next
    }

    pub fn is_done(&self) -> bool {
        self.next >= self.input.len()
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        let mut col = 1;
        let mut ln = 1;

        for (i, ch) in self.input.iter().enumerate() {
            if self.next + 1 == i {
                break;
            }
            if ch == &'\n' {
                ln = 1;
                col += 1;
            } else if ch == &'\r' {
                continue
            } else {
                ln += 1;
            }
        }
        (col, ln)
    }

    /// Resets `Muncher.peek` to current `Muncher.next`
    pub fn reset_peek(&self) -> usize {
        self.peek.set(self.next);
        self.peek.get()
    }

    /// increments `Muncher.peek` by one
    fn adv_peek(&self) -> usize {
        let inc = self.peek.get();
        self.peek.set(inc + 1);
        inc
    }

    /// Gets the char at `Muncher.peek` index then increments `Muncher.peek` by one
    pub fn peek(&self) -> Option<&char> {
        let res = self.input.get(self.peek.get());
        self.adv_peek();
        res
    }

    /// Peek tokens until given predicate is true.
    /// Resets the peek position every time called.
    pub fn peek_until<P>(&self, mut pred: P) -> impl Iterator<Item = &char>
    where
        P: FnMut(&char) -> bool,
    {
        let start = self.peek.get();
        for ch in self.input[start..].iter() {
            if pred(ch) {
                break;
            } else {
                self.peek.set(self.peek.get() + 1);
            }
        }
        let end = self.peek.get();
        self.peek.set(end);
        self.input[start..end].iter()
    }

    /// Peek tokens until given predicate is true returns start and end.
    /// Resets the peek position every time called.
    /// 
    /// # Example
    /// ```
    /// use toml_parse::Muncher;
    /// 
    /// let input = "abcde";
    /// let mut munch = Muncher::new(input);
    /// 
    /// let (start, end) = munch.peek_until_count(|ch| ch == &'d');
    /// assert_eq!(&munch.text()[start..end], "abc");
    /// ``` 
    pub fn peek_until_count<P>(&self, mut pred: P) -> (usize, usize)
    where
        P: FnMut(&char) -> bool,
    {
        let start = self.reset_peek();
        for ch in self.input[start..].iter() {
            if pred(ch) {
                break;
            } else {
                self.peek.set(self.peek.get() + 1);
            }
        }
        let end = self.peek.get();
        (start, end)
    }

    /// Peeks tokens until needle is found returns start and end.
    /// Resets the peek position every time called.
    /// # Example
    /// ```
    /// use toml_parse::Muncher;
    /// 
    /// let input = "abcde";
    /// let mut munch = Muncher::new(input);
    /// 
    /// let (start, end) = munch.peek_range_of("d");
    /// assert_eq!(&munch.text()[start..end], "abc");
    /// ```
    pub fn peek_range_of(&self, needle: &str) -> (usize, usize) {
        let start = self.reset_peek();
        let split = self.text[start..].split(needle).collect::<Vec<_>>();
        let end = start + split[0].chars().count();
        (start, end)
    }

    /// Returns `Some(&str)` if `seek` does not run into the end
    /// of the input.
    /// 
    /// # Example
    /// ```
    /// use toml_parse::Muncher;
    /// 
    /// let input = "hello world";
    /// let m = Muncher::new(input);
    /// assert_eq!(m.seek(5), Some("hello"));
    /// ```
    pub fn seek(&self, count: usize) -> Option<&str> {
        let start = self.peek.get();
        let end = start + count;
        if end > self.input.len() {
            return None;
        }
        self.peek.set(end);
        Some(&self.text[start..end])
    }

    /// Eats the next char if not at end of input
    /// 
    /// # Example
    /// ```
    /// use toml_parse::Muncher;
    /// 
    /// let input = "abc";
    /// let mut m = Muncher::new(input);
    /// assert_eq!(m.eat(), Some('a'));
    /// assert_eq!(m.eat(), Some('b'));
    /// assert_eq!(m.eat(), Some('c'));
    /// assert_eq!(m.eat(), None);
    /// ```
    pub fn eat(&mut self) -> Option<char> {
        let res = self.input.get(self.next).copied();
        self.next += 1;
        self.peek.set(self.next);
        res
    }

    /// Eats next white space if next char is space and returns true.
    /// 
    pub fn eat_ws(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&' ') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }
    /// Eats next newline if next char is newline and returns true.
    /// This handles both windows and unix line endings.
    /// 
    pub fn eat_eol(&mut self) -> bool {
        self.reset_peek();
        let next = self.peek();
        if next == Some(&'\n') {
            self.eat().is_some()
        } else if next == Some(&'\r') {
            self.eat();
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }
    /// Eats equal sign and returns true, false if not found
    pub fn eat_eq(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'=') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_open_brc(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'[') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_close_brc(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&']') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_open_curly(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'{') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_close_curly(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'}') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_double_quote(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'"') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_single_quote(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'\'') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_comma(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&',') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_hash(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'#') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_plus(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'+') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_minus(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'-') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_colon(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&':') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    pub fn eat_dot(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'.') {
            self.eat().is_some()
        } else {
            self.reset_peek();
            false
        }
    }

    /// Eat tokens until given predicate is true.
    pub fn eat_until<P>(&mut self, mut pred: P) -> impl Iterator<Item = char>
    where
        P: FnMut(&char) -> bool,
    {
        let start = self.next;
        for ch in self.input[start..].iter() {
            if pred(ch) {
                break;
            } else {
                self.next += 1;
            }
        }
        let end = self.next;
        self.peek.set(end);
        self.next = end;

        self.input[start..end]
            .iter()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn eat_until_count<P>(&mut self, mut pred: P) -> (usize, usize)
    where
        P: FnMut(&char) -> bool,
    {
        let start = self.next;
        for ch in self.input[start..].iter() {
            if pred(ch) {
                break;
            } else {
                self.next += 1;
            }
        }
        let end = self.next;
        self.peek.set(end);
        self.next = end;

        (start, end)
    }

    /// Eat tokens until needle is found returns start and end.
    /// Resets the peek position every time called.
    pub fn eat_range_of(&mut self, needle: &str) -> (usize, usize) {
        self.reset_peek();
        let start = self.next;
        let split = self.text[start..].split(needle).collect::<Vec<_>>();

        let end = start + split[0].chars().count();
        self.next = end;
        (start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_muncher() {
        let input = "hello world";
        let mut m = Muncher::new(input);

        assert_eq!(m.eat(), Some('h'));

        for ch in m.eat_until(|c| c.is_whitespace()) {
            assert!(!ch.is_whitespace());
        }
        assert_eq!(m.peek(), Some(&' '));
        assert_eq!(m.eat(), Some(' '));
    }

    #[test]
    fn end_eat_while_muncher() {
        let input = "hello world";
        let mut m = Muncher::new(input);

        assert_eq!(m.eat(), Some('h'));

        for ch in m.eat_until(|c| c.is_whitespace()) {
            assert!(!ch.is_whitespace());
        }
        assert_eq!(m.peek(), Some(&' '));
        assert_eq!(m.eat(), Some(' '));
        for ch in m.eat_until(|c| c.is_whitespace()) {
            assert!(!ch.is_whitespace());
        }
        assert!(m.eat().is_none());
        assert!(m.peek().is_none());
    }

    #[test]
    fn peek_muncher() {
        let input = "hello world";
        let chars = input.to_string().chars().collect::<Vec<char>>();
        let mut m = Muncher::new(input);

        assert_eq!(m.eat(), Some('h'));

        let mut idx = 0;
        while let Some(_ch) = m.eat() {
            idx += 1;
            assert_eq!(m.peek(), chars.get(idx + 1));
        }
    }

    #[test]
    fn peek_count() {
        let input = "abcde";
        let mut munch = Muncher::new(input);
        
        let (start, end) = munch.peek_until_count(|ch| ch == &'d');
        assert_eq!(&munch.text()[start..end], "abc");
    }

    #[test]
    fn peek_range_of() {
        let input = "abcde";
        let mut munch = Muncher::new(input);
        
        let (start, end) = munch.peek_range_of("d");
        assert_eq!(&munch.text()[start..end], "abc");
    }

    #[test]
    fn seek_muncher() {
        let input = "hello world";
        let m = Muncher::new(input);

        assert_eq!(m.seek(5), Some("hello"));
        assert_eq!(m.peek(), Some(&' '));
        println!("{:#?}", m);
        assert_eq!(m.seek(5), Some("world"));
        assert!(m.peek().is_none());
    }

    #[test]
    fn test_eat_eol() {
        let input = "hello\nworld";
        let mut m = Muncher::new(input);

        // this will advance the cursor.
        // this may not further allocate?
        m.eat_until(|c| c == &'\n');
        assert_eq!(m.peek(), Some(&'\n'));
        assert!(m.eat_eol());
    }

    #[test]
    fn test_fork() {
        let input = "abcde";
        let mut munch = Muncher::new(input);
        assert_eq!(munch.eat(), Some('a'));
        
        let fork = munch.fork();
        assert_eq!(fork.peek(), Some(&'b'));
        assert_eq!(munch.eat(), Some('b'));
        assert_eq!(munch.eat(), Some('c'));
    }
}

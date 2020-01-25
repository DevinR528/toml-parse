use std::cell::Cell;

use super::err::ParseTomlError;
use super::Value;

pub fn cmp_tokens(ch: &char, chars: &[char]) -> bool {
    let res = chars.iter().any(|c| c == ch);
    println!("{} {:?}", res, ch);
    res
}

#[derive(Debug, Clone)]
pub struct Fork<'a> {
    input: &'a [char],
    peek: Cell<usize>,
}

impl<'f> Fork<'f> {
    pub(crate) fn new(input: &'f [char]) -> Self {
        Self { 
            input,
            peek: Cell::new(0),
         }
    }

    pub(crate) fn reset_peek(&self) {
        self.peek.set(0);
    }

    fn adv_peek(&self) -> usize {
        self.peek.get() + 1
    }

    pub(crate) fn peek(&self) -> Option<&char> {
        self.input.get(self.adv_peek())
    }

    pub(crate) fn seek(&self, count: usize) -> Option<String> {
        let start = self.peek.get();
        let end = start + count;
        if end >= self.input.len() {
            return None;
        }
        Some(self.input[start..end].iter().collect())
    }

    /// Peek tokens while predicate is true.
    pub(crate) fn peek_while<P>(&self, pred: P) -> impl Iterator<Item=&char>
    where
        P: Fn(&char) -> bool,
    {
        for ch in self.input.iter() {
            if pred(ch) && self.peek.get() != self.input.len() {
                self.peek.set(self.peek.get() + 1);
            } else {
                break;
            }
        }
        let end = self.peek.get();
        self.reset_peek();
        self.input.windows(end).next().map(|chunk| chunk.iter()).unwrap_or_else(|| [].iter())

    }

    /// Peek tokens until given predicate is true.
    pub(crate) fn peek_until<P>(&self, pred: P) -> impl Iterator<Item=&char>
    where
        P: Fn(&char) -> bool,
    {
        for ch in self.input.iter() {
            if pred(ch) {
                break;
            } else {
                self.peek.set(self.peek.get() + 1);
            }
        }
        let end = self.peek.get();
        self.reset_peek();
        self.input.windows(end).next().map(|chunk| chunk.iter()).unwrap_or_else(|| [].iter())
    }  
}

#[derive(Debug, Clone)]
pub struct Muncher {
    input: Vec<char>,
    peek: Cell<usize>,
    next: usize,
}

impl Muncher {
    pub(crate) fn new(input: &str) -> Self {
        Self { 
            input: input.chars().collect(),
            peek: Cell::new(0),
            next: 0,
         }
    }

    pub(crate) fn fork(&self) -> Fork {
        Fork {
            input: &self.input[self.next..],
            peek: Cell::new(0),
        }
    }

    pub(crate) fn position(&self) -> usize {
        self.next - 1
    }

    pub(crate) fn is_done(&self) -> bool {
        self.next >= self.input.len()
    }

    /// Resets `Muncher.peek` to current `Muncher.next`
    fn reset_peek(&self) -> usize {
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
    pub(crate) fn peek(&self) -> Option<&char> {
        let res = self.input.get(self.peek.get());
        self.adv_peek();
        res
    }
    
    pub(crate) fn seek(&self, count: usize) -> Option<String> {
        let start = self.peek.get();
        self.peek.set(start + count);
        if self.peek.get() + 1 == self.input.len() {
            return None;
        }
        Some(self.input[start..self.peek.get()].iter().collect())
    }

    /// Peek tokens while predicate is true.
    pub(crate) fn peek_while<P>(&self, pred: P) -> impl Iterator<Item=&char>
    where
        P: Fn(&char) -> bool,
    {
        let start = self.reset_peek();
        for ch in self.input[start..].iter() {
            if pred(ch) && self.peek.get() != self.input.len() {
                self.adv_peek();
            } else {
                if self.peek.get() == self.input.len() {
                    self.peek.set(self.peek.get() + 1);
                }
                break;
            }
        }
        let end = self.peek.get();
        self.peek.set(end);
        self.input[start..end].iter()
    }

    /// Peek tokens until given predicate is true.
    pub(crate) fn peek_until<P>(&self, pred: P) -> impl Iterator<Item=&char>
    where
        P: Fn(&char) -> bool,
    {
        let start = self.reset_peek();
        println!("start {} {:?}", start, self.input.get(start));
        for ch in self.input[start..].iter() {
            if pred(ch) {
                if self.peek.get() == self.input.len() {
                    self.peek.set(self.peek.get() + 1);
                }
                break;
            } else {
                self.adv_peek();
            }
        }
        let end = self.peek.get();
        println!("start {} {:?}", end, self.input.get(end));
        self.peek.set(end);
        self.input[start..end].iter()
    } 

    pub(crate) fn eat(&mut self) -> Option<char> {
        let res = self.input.get(self.next).copied();
        self.next += 1;
        self.peek.set(self.next);
        res
    }

    pub(crate) fn eat_ws(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&' ') {
            self.eat().is_some()
        } else {
            false
        }
    }

    pub(crate) fn eat_eol(&mut self) -> bool {
        self.reset_peek();
        let next = self.peek();
        if next == Some(&'\n') {
            self.eat().is_some()
        } else if next == Some(&'\r') {
            self.eat();
            self.eat().is_some()
        } else {
            false
        }
    }

    pub(crate) fn eat_eq(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'=') {
            let res = self.eat().is_some();
            self.eat_ws();
            res
        } else {
            false
        }
    }

    pub(crate) fn eat_open_brc(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'[') {
            self.eat().is_some()
        } else {
            false
        }
    }

    pub(crate) fn eat_close_brc(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&']') {
            self.eat().is_some()
        } else {
            false
        }
    }

    pub(crate) fn eat_open_curly(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'{') {
            self.eat().is_some()
        } else {
            false
        }
    }

    pub(crate) fn eat_close_curly(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'}') {
            self.eat().is_some()
        } else {
            false
        }
    }

    pub(crate) fn eat_quote(&mut self) -> bool {
        self.reset_peek();
        if self.peek() == Some(&'"') {
            self.eat().is_some()
        } else {
            false
        }
    }

    /// Eat tokens while predicate is true.
    pub(crate) fn eat_while<P>(&mut self, pred: P) -> impl Iterator<Item=char>
    where
        P: Fn(&char) -> bool,
    {
        let start = self.next;
        for ch in self.input[start..].iter() {
            if pred(ch) && self.next != self.input.len() {
                self.next += 1;
            } else {
                if self.next == self.input.len() {
                    self.next += 1;
                }
                break;
            }
        }
        let end = self.next;
        self.peek.set(end);
        self.next = end;
        self.input[start..end].iter().copied().collect::<Vec<_>>().into_iter()
    }

    /// Eat tokens until given predicate is true.
    pub(crate) fn eat_until<P>(&mut self, mut pred: P) -> impl Iterator<Item=char>
    where
        P: FnMut(&char) -> bool,
    {
        let start = self.next;
        for ch in self.input[start..].iter() {
            if pred(ch) {
                if self.next == self.input.len() {
                    return vec![].into_iter()
                }
                break;
            } else {
                self.next += 1;
            }
        }
        let end = self.next;
        self.peek.set(end);
        self.next = end;
        println!("eat until ({}, {}) {:?}", start, end, &self.input[start..end]);
        self.input[start..end].iter().copied().collect::<Vec<_>>().into_iter()
    } 
}


#[derive(Debug, Clone)]
pub struct TomlTokenizer {
    pub tables: Vec<Value>,
    inner: Muncher,
}

impl TomlTokenizer {
    pub fn new(input: &str) -> TomlTokenizer {
        Self {
            inner: Muncher::new(input),
            tables: Vec::default(),
        }
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

        for ch in m.eat_while(|c| !c.is_whitespace()) {
            assert!(!ch.is_whitespace());
        }
        assert_eq!(m.peek(), Some(&' '));
        assert_eq!(m.eat(), Some(' '));
        for ch in m.eat_while(|c| !c.is_whitespace()) {
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
        while let Some(ch) = m.eat() {
            idx += 1;
            assert_eq!(m.peek(), chars.get(idx + 1));
        }
    }

    #[test]
    fn peek_while_muncher() {
        let input = "hello world";
        let chars = input.to_string().chars().collect::<Vec<char>>();
        let mut m = Muncher::new(input);

        assert_eq!(m.eat(), Some('h'));

        for ch in m.peek_until(|c| c.is_whitespace()) {
            assert!(!ch.is_whitespace());
        }

        assert_eq!(m.peek(), Some(&' '));

        for ch in m.peek_until(|c| c.is_whitespace()) {
            assert!(!ch.is_whitespace());
        }
    }

    #[test]
    fn seek_muncher() {
        let input = "hello world";
        let chars = input.to_string().chars().collect::<Vec<char>>();
        let m = Muncher::new(input);

        assert_eq!(m.seek(5), Some("hello".to_string()));
        assert_eq!(m.peek(), Some(&' '));
        println!("{:#?}", m);
        assert_eq!(m.seek(5), Some("world".to_string()));
        assert!(m.peek().is_none());
    }

    #[test]
    fn test_eat_eol() {
        let input = "hello\nworld";
        let mut m = Muncher::new(input);

        // this will advance the cursor.
        // this may not further allocat?
        m.eat_until(|c| c == &'\n');
        assert_eq!(m.peek(), Some(&'\n'));
        assert!(m.eat_eol());
    }
}

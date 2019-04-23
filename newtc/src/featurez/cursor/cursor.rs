use std::str::Chars;

pub struct Cursor<'a> {
    text: &'a str,
    len: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(text: &'a str) -> Cursor<'a> {
        Cursor { text, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn current(&self) -> Option<char> {
        self.chars().next()
    }

    pub fn peek(&self, n: usize) -> Option<char> {
        self.chars().nth(n)
    }

    pub fn current_token_text(&self) -> &str {
        &self.text[..self.len]
    }

    pub fn match_char(&self, c: char) -> bool {
        self.current() == Some(c)
    }

    pub fn match_str(&self, s: &str) -> bool {
        self.chars().as_str().starts_with(s)
    }

    pub fn match_char_predicate<P: Fn(char) -> bool>(&self, predicate: P) -> bool {
        self.current().map(predicate) == Some(true)
    }

    pub fn match_nth_predicate<P: Fn(char) -> bool>(&self, n: usize, predicate: P) -> bool {
        self.peek(n).map(predicate) == Some(true)
    }

    fn chars(&self) -> Chars {
        self.text[self.len..].chars()
    }
}

impl<'a> Iterator for Cursor<'a> where {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let next = self.chars().next()?;
        self.len += 1;
        Some(next)
    }
}

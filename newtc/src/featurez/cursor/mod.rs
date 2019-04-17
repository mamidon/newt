use std::str::Chars;

pub struct Cursor<'a>
{
	text: &'a str,
	len: usize
}


impl<'a> Cursor<'a> 
{
	pub fn new(text: &'a str) -> Cursor<'a> {
		Cursor {
			text,
			len: 0
		}
	}
	
	pub fn len(&self) -> usize { self.len }

	pub fn current(&self) -> Option<char> { self.chars().next() }
	
	pub fn peek(&self, n: usize) -> Option<char> { self.chars().nth(n) }
	
	pub fn current_token_text(&self) -> &str { &self.text[..self.len] }
	
	pub fn match_char(&self, c: char) -> bool { self.current() == Some(c) }
	
	pub fn match_str(&self, s: &str) -> bool { self.chars().as_str().starts_with(s) }
	
	pub fn match_char_predicate<P: Fn(char) -> bool>(&self, predicate: P) -> bool 
	{ 
		self.current().map(predicate) == Some(true)
	}
	
	pub fn match_nth_predicate<P: Fn(char) -> bool>(&self, n: usize, predicate: P) -> bool 
	{
		self.peek(n).map(predicate) == Some(true)
	}
	
	fn chars(&self) -> Chars { self.text[self.len..].chars() }
}

impl<'a> Iterator for Cursor<'a> where
{
	type Item = char;

	fn next(&mut self) -> Option<char> {
		let next = self.chars().next()?;
		self.len += 1;
		Some(next)
	}
}


#[cfg(test)]
mod tests {
	use super::{Cursor};
	
	#[test]
	fn cursor_next_iterates() {
		let mut cursor = Cursor::new("test");
		
		assert_eq!(cursor.next(), Some('t'));
		assert_eq!(cursor.next(), Some('e'));
		assert_eq!(cursor.next(), Some('s'));
		assert_eq!(cursor.next(), Some('t'));
		assert_eq!(cursor.next(), None);
	}

	#[test]
	fn cursor_current_gets_next_char_without_iterating() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.current(), Some('t'));
		assert_eq!(cursor.current(), Some('t'));
		cursor.next();
		assert_eq!(cursor.current(), Some('e'));
		cursor.next();
		assert_eq!(cursor.current(), Some('s'));
		assert_eq!(cursor.current(), Some('s'));
		cursor.next();
		assert_eq!(cursor.current(), Some('t'));
		cursor.next();
		assert_eq!(cursor.next(), None);
	}

	#[test]
	fn cursor_peek_gets_nth_item() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.peek(0), Some('t'));
		assert_eq!(cursor.peek(1), Some('e'));
		assert_eq!(cursor.peek(2), Some('s'));
		assert_eq!(cursor.peek(3), Some('t'));
		assert_eq!(cursor.peek(4), None);
	}

	#[test]
	fn cursor_peek_does_not_consume_items() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.current(), Some('t'));
		assert_eq!(cursor.len(), 0);
		assert_eq!(cursor.peek(1), Some('e'));
		assert_eq!(cursor.current(), Some('t'));
		assert_eq!(cursor.len(), 0);
	}

	#[test]
	fn cursor_peek_is_relative_to_current_item() {
		let mut cursor = Cursor::new("test");

		cursor.next();
		assert_eq!(cursor.peek(1), Some('s'));
	}

	#[test]
	fn cursor_len_counts_consumed_characters() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.len(), 0);
		cursor.next();
		assert_eq!(cursor.len(), 1);
		cursor.next();
		assert_eq!(cursor.len(), 2);
		cursor.next();
		assert_eq!(cursor.len(), 3);
		cursor.next();
		assert_eq!(cursor.len(), 4);
		cursor.next();
		assert_eq!(cursor.len(), 4);
	}
	
	#[test]
	fn cursor_current_token_text_gets_current_lexeme() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.current_token_text(), "");
		cursor.next();

		assert_eq!(cursor.current_token_text(), "t");
		cursor.next();

		assert_eq!(cursor.current_token_text(), "te");
		cursor.next();

		assert_eq!(cursor.current_token_text(), "tes");
		cursor.next();

		assert_eq!(cursor.current_token_text(), "test");
		cursor.next();

		assert_eq!(cursor.current_token_text(), "test");
		cursor.next();
	}
	
	#[test]
	fn cursor_match_char_compares_current() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.match_char('t'), true);
		assert_eq!(cursor.match_char('e'), false);
	}

	#[test]
	fn cursor_match_char_predicate_tests_current() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.match_char_predicate(|c: char| c == 't'), true);
		assert_eq!(cursor.match_char_predicate(|c: char| c == 'e'), false);
	}

	#[test]
	fn cursor_match_nth_predicate_tests_nth() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.match_nth_predicate(1, |c: char| c == 't'), false);
		assert_eq!(cursor.match_nth_predicate(1, |c: char| c == 'e'), true);
	}
	
	#[test]
	fn cursor_match_str_compares_prefix() {
		let mut cursor = Cursor::new("test");

		assert_eq!(cursor.match_str("t"), true);
		assert_eq!(cursor.match_str("te"), true);
		assert_eq!(cursor.match_str("tes"), true);
		assert_eq!(cursor.match_str("test"), true);
		
		assert_eq!(cursor.match_str("test "), false);
		assert_eq!(cursor.match_str("est"), false);
	}
}

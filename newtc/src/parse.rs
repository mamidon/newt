use std::str::Chars;

enum TokenKind {
	Let,
	Identifier,
	Literal,
	Plus,
	Semicolon
}

struct Token {
	token_type: TokenKind,
	length: usize
}

struct Cursor<'a> {
	text: &'a str,
	consumed: usize
}

impl<'a> Cursor<'a> {
	fn new(text: &'a str) -> Cursor {
		Cursor {
			text,
			consumed: 0
		}
	}
	
	fn current(&self) -> Option<char> {
		self.chars().next()
	}
	
	fn nth(&self, n: usize) -> Option<char> {
		self.chars().nth(n)
	}
	
	fn consume(&mut self) -> Option<char> {
		let next = self.current()?;
		self.consumed += 1;
		Some(next)
	}
	
	fn matches(&self, candidate: char) -> bool {
		self.current() == candidate
	} 
	
	fn matches_predicate<P: Fn(char) -> bool>(&self, predicate: P) -> bool {
		self.current().map(predicate).is_some()
	}
	
	fn chars(&self) -> Chars {
		self.text[self.consumed..].chars()
	}
}
    
fn lex(text: &str) -> Vec<Token> {
	let mut tokens: Vec<Token> = vec![];
	let mut cursor = Cursor::new(text);
	
	
}
/*
keywords: 

	let x = 3 + 4;
	
*/
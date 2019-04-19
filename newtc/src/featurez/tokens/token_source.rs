use crate::featurez::tokens::{Token, TokenKind};
use crate::featurez::syntax::TokenSource;

pub struct StrTokenSource<'a> {
	text: &'a str,
	tokens: Vec<Token>,
	offsets: Vec<usize>
}

impl<'a> StrTokenSource<'a> {
	pub fn new(text: &'a str, raw_tokens: Vec<Token>) -> StrTokenSource {
		let mut tokens: Vec<Token> = vec![];
		let mut offsets: Vec<usize> = vec![];
		let mut length: usize = 0;

		for token in raw_tokens.iter() {
			tokens.push(*token);
			offsets.push(length);
			length += token.lexeme_length();
		}

		StrTokenSource {
			text,
			tokens,
			offsets
		}
	}
}

impl<'a> TokenSource for StrTokenSource<'a> {
	fn token(&self, index: usize) -> Token {
		if index >= self.tokens.len() {
			Token::end_of_file()
		} else {
			self.tokens[index]
		}
	}

	fn token_kind(&self, index: usize) -> TokenKind {
		if index >= self.tokens.len() {
			TokenKind::EndOfFile
		} else {
			self.tokens[index].token_kind()
		}
	}

	fn token2(&self, pos: usize) -> Option<(Token, Token)> {
		if pos + 1 < self.tokens.len() {
			Some((self.tokens[pos], self.tokens[pos+1]))
		} else {
			None
		}
	}

	fn token3(&self, pos: usize) -> Option<(Token, Token, Token)> {
		let remaining = self.tokens.len() - pos;
		if remaining < 3 {
			None
		} else {
			Some((self.tokens[pos], self.tokens[pos+1], self.tokens[pos+2]))
		}
	}
}
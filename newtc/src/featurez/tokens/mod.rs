use super::cursor::{Cursor};
use super::syntax::TokenSource;
pub use self::token_source::StrTokenSource;

use std::fmt::{Display, Formatter};
use std::fmt::Error;
use std::fmt::Debug;

mod token_source;
mod tests;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum TokenKind {
	WhiteSpace,
	CommentLine,
	CommentBlock,

	// single character tokens

	// grouping tokens
	LeftBrace,
	RightBrace,
	LeftParenthesis,
	RightParenthesis,
	LeftBracket,
	RightBracket,

	Comma,
	Dot,
	Colon,
	SemiColon,
	UnderScore,

	// math, comparison, and logic operators
	Equals,
	EqualsEquals,
	Plus,
	Minus,
	Star,
	Slash,
	Greater,
	GreaterEquals,
	Less,
	LessEquals,
	Ampersand,
	AmpersandAmpersand,
	Pipe,
	PipePipe,
	Bang,

	// literals
	IntegerLiteral,
	FloatLiteral,
	StringLiteral,
	GlyphLiteral,

	Identifier,

	// keywords
	Fn,
	Return,
	If,
	Else,
	For,
	In,
	While,
	Let,
	True,
	False,

	EndOfFile,
	TombStone
}

#[derive(Copy, Clone)]
pub struct Token {
	token_type: TokenKind,
	length: usize,
}

impl Token {
	fn new(token_type: TokenKind, length: usize) -> Token {
		Token {
			token_type,
			length
		}
	}

	fn merge_as(token_type: TokenKind, left: &Token, right: &Token) -> Token {
		Token {
			token_type,
			length: left.length + right.length
		}
	}
	
	pub fn tomb_stone() -> Token { Token::new(TokenKind::TombStone, 0) }
	
	pub fn end_of_file() -> Token { Token::new(TokenKind::EndOfFile, 0) }

	pub fn token_type(&self) -> TokenKind {
		self.token_type
	}

	pub fn lexeme_length(&self) -> usize {
		self.length
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{:?}[{}]", self.token_type, self.length)
	}
}

impl Debug for Token {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{:?}[{}]", self.token_type, self.length)
	}
}

impl PartialEq for Token {
	fn eq(&self, other: &Token) -> bool {
		self.token_type == other.token_type && self.length == other.length
	}
}

pub fn tokenize(text: &str) -> Vec<Token> {
	let mut tokens: Vec<Token> = vec![];
	let mut source = text;
	
	while !source.is_empty() {
		let mut token = next_token(source);
		
		tokens.push(token);
		source = &source[token.length..];
	}
	
	tokens.push(Token::new(TokenKind::EndOfFile, 0));
	tokens
}

fn next_token(text: &str) -> Token {
	let mut cursor = Cursor::new(text);
	
	if let Some(token) = lex_whitespace(&mut cursor) {
		token
	} else if let Some(token) = lex_multi_character_token(&mut cursor) {
		token
	} else if let Some(token) = lex_two_character_token(&mut cursor) {
		token
	} else if let Some(token) = lex_single_character_token(&mut cursor) {
		token
	} else {
		cursor;
		Token::new(TokenKind::TombStone, 1)
	}
}


fn lex_whitespace(cursor: &mut Cursor) -> Option<Token> {
	while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c.is_whitespace()) {
		cursor.next();
	}

	if cursor.len() > 0 {
		Some(Token::new(TokenKind::WhiteSpace, cursor.len()))
	} else {
		None
	}
}


fn lex_single_character_token(cursor: &mut Cursor) -> Option<Token> {
	fn make_token(token_type: TokenKind) -> Token {
		Token::new(token_type, 1)
	}

	let current = cursor.current();
	
	let token = match current {
		Some('{') => make_token(TokenKind::LeftBrace),
		Some('}') => make_token(TokenKind::RightBrace),
		Some('(') => make_token(TokenKind::LeftParenthesis),
		Some(')') => make_token(TokenKind::RightParenthesis),
		Some('[') => make_token(TokenKind::LeftBracket),
		Some(']') => make_token(TokenKind::RightBracket),

		Some(',') => make_token(TokenKind::Comma),
		Some('.') => make_token(TokenKind::Dot),
		Some(':') => make_token(TokenKind::Colon),
		Some(';') => make_token(TokenKind::SemiColon),
		Some('_') => make_token(TokenKind::UnderScore),

		Some('=') => make_token(TokenKind::Equals),
		Some('+') => make_token(TokenKind::Plus),
		Some('-') => make_token(TokenKind::Minus),
		Some('*') => make_token(TokenKind::Star),
		Some('/') => make_token(TokenKind::Slash),
		Some('>') => make_token(TokenKind::Greater),
		Some('<') => make_token(TokenKind::Less),

		Some('|') => make_token(TokenKind::Pipe),
		Some('&') => make_token(TokenKind::Ampersand),
		Some('!') => make_token(TokenKind::Bang),

		_ => return None
	};
	
	cursor.next();

	Some(token)
}


fn lex_two_character_token(cursor: &mut Cursor) -> Option<Token> {
	fn make_token(cursor: &mut Cursor, token_type: TokenKind) -> Token {
		cursor.next();
		cursor.next();

		Token::new(token_type, 2)
	}

	fn make_token_consume_line(cursor: &mut Cursor, token_type: TokenKind) -> Token {
		let mut length = 2;
		cursor.next();
		cursor.next();

		while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c != '\n') {
			length += 1;
			cursor.next();
		}

		Token::new(token_type, length)
	}
	
	let current = cursor.current();
	let next = cursor.peek(1);
	
	if let (Some(current), Some(next)) = (cursor.current(), cursor.peek(1)) {
		let token = match (current, next) {
			('=', '=') => make_token(cursor, TokenKind::EqualsEquals),
			('>', '=') => make_token(cursor, TokenKind::GreaterEquals),
			('<', '=') => make_token(cursor, TokenKind::LessEquals),
			('|', '|') => make_token(cursor, TokenKind::PipePipe),
			('&', '&') => make_token(cursor, TokenKind::AmpersandAmpersand),
			('/', '/') => make_token_consume_line(cursor, TokenKind::CommentLine),
			_ => return None
		};

		Some(token)
	} else {
		None
	}
}



fn lex_multi_character_token(cursor: &mut Cursor) -> Option<Token> {
	if let Some(numeric_literal) = scan_numeric_literal(cursor) {
		Some(numeric_literal)
	} else if let Some(string) = scan_string_literal(cursor) {
		Some(string)
	} else if let Some(glyph) = scan_glyph_literal(cursor) {
		Some(glyph)
	} else if let Some(identifier) = scan_identifier(cursor) {
		Some(identifier)
	} else {
		None
	}
}

fn scan_identifier(cursor: &mut Cursor) -> Option<Token> {
	let starting_predicate = |c: char| c.is_alphabetic() || c == '_';
	let suffix_predicate = |c: char| c.is_alphanumeric() || c == '_';

	if !cursor.match_char_predicate(&starting_predicate) {
		return None;
	}

	let mut lexeme = String::new();
	while cursor.current().is_some() && cursor.match_char_predicate(&suffix_predicate) {
		lexeme.push(cursor.next().unwrap());
	}

	if lexeme.len() == 1 && lexeme.starts_with('_') {
		Some(Token::new(TokenKind::UnderScore, cursor.len()))
	} else if let Some(keyword) = match_identifier_to_keyword(&lexeme) {
		Some(Token::new(keyword, cursor.len()))
	} else {
		Some(Token::new(TokenKind::Identifier, cursor.len()))
	}
}

fn scan_string_literal(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.match_char('"') {
		return None;
	}

	cursor.next();
	cursor.next();

	while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c != '"') {
		cursor.next();
	}

	if cursor.match_char('"') {
		cursor.next();
		Some(Token::new(TokenKind::StringLiteral, cursor.len()))
	} else {
		Some(Token::new(TokenKind::TombStone, cursor.len()))
	}
}

fn scan_glyph_literal(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.match_char('\'') {
		return None;
	}

	cursor.next();
	cursor.next();

	while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c != '\'') {
		cursor.next();
	}

	if cursor.match_char('\'') {
		cursor.next();
		Some(Token::new(TokenKind::GlyphLiteral, cursor.len()))
	} else {
		Some(Token::new(TokenKind::TombStone, cursor.len()))
	}
}

fn scan_numeric_literal(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.match_char_predicate(&|c: char| c.is_digit(10)) {
		return None;
	}
	
	while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c.is_digit(10)) {
		cursor.next();
	}

	if cursor.match_char('.') {
		cursor.next();

		while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c.is_digit(10)) {
			cursor.next();
		}

		Some(Token::new(TokenKind::FloatLiteral, cursor.len()))
	} else {
		Some(Token::new(TokenKind::IntegerLiteral, cursor.len()))
	}
}


fn match_identifier_to_keyword(lexeme: &str) -> Option<TokenKind> {
	match lexeme.to_lowercase().as_str() {
		"fn" => Some(TokenKind::Fn),
		"return" => Some(TokenKind::Return),
		"if" => Some(TokenKind::If),
		"else" => Some(TokenKind::Else),
		"for" => Some(TokenKind::For),
		"in" => Some(TokenKind::In),
		"while" => Some(TokenKind::While),
		"let" => Some(TokenKind::Let),
		"true" => Some(TokenKind::True),
		"false" => Some(TokenKind::False),
		_ => None
	}
}

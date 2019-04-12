use crate::parse::{Cursor};

use std::fmt::{Display, Formatter};
use std::fmt::Error;

mod tests;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
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
	token_type: TokenType,
	length: usize,
}

impl Token {
	fn new(token_type: TokenType, length: usize) -> Token {
		Token {
			token_type,
			length
		}
	}

	fn merge_as(token_type: TokenType, left: &Token, right: &Token) -> Token {
		Token {
			token_type,
			length: left.length + right.length
		}
	}

	pub fn token_type(&self) -> TokenType {
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

pub fn tokenize(text: &str) -> Vec<Token> {
	let mut tokens: Vec<Token> = vec![];
	let mut cursor = Cursor::new(text);

	while cursor.current().is_some() {
		let token = next_token(&mut cursor);

		if token.token_type == TokenType::TombStone {
			while tokens.last().filter(|t| t.token_type == TokenType::TombStone).is_some() {
				let preceding_token = tokens.last().expect("We checked the token exists in the while statement");
				let merged_token = Token::merge_as(TokenType::TombStone, preceding_token, &token);
				tokens.pop();
			}
		}

		tokens.push(token);
	}

	tokens.push(Token::new(TokenType::EndOfFile, 0));
	tokens
}

fn merge_adjacent_tokens(token_type: TokenType, preceding_token: Token, tombstone: Token) -> Token {
	Token::new(token_type, preceding_token.length + tombstone.length)
}

fn next_token(cursor: &mut Cursor) -> Token {
	if let Some(token) = lex_whitespace(cursor) {
		token
	} else if let Some(token) = lex_multi_character_token(cursor) {
		token
	} else if let Some(token) = lex_two_character_token(cursor) {
		token
	} else if let Some(token) = lex_single_character_token(cursor) {
		token
	} else {
		cursor.consume();
		Token::new(TokenType::TombStone, 1)
	}
}

fn lex_whitespace(cursor: &mut Cursor) -> Option<Token> {
	let offset = cursor.consumed;

	while !cursor.empty() && cursor.matches_predicate(|c| c.is_whitespace()) {
		cursor.consume();
	}

	if offset != cursor.consumed {
		Some(Token::new(TokenType::WhiteSpace, cursor.consumed - offset))
	} else {
		None
	}
}

fn lex_single_character_token(cursor: &mut Cursor) -> Option<Token> {
	fn make_token(token_type: TokenType) -> Token {
		Token::new(token_type, 1)
	}

	let current = cursor.current();

	let token = match current {
		Some('{') => make_token(TokenType::LeftBrace),
		Some('}') => make_token(TokenType::RightBrace),
		Some('(') => make_token(TokenType::LeftParenthesis),
		Some(')') => make_token(TokenType::RightParenthesis),
		Some('[') => make_token(TokenType::LeftBracket),
		Some(']') => make_token(TokenType::RightBracket),

		Some(',') => make_token(TokenType::Comma),
		Some('.') => make_token(TokenType::Dot),
		Some(':') => make_token(TokenType::Colon),
		Some(';') => make_token(TokenType::SemiColon),
		Some('_') => make_token(TokenType::UnderScore),

		Some('=') => make_token(TokenType::Equals),
		Some('+') => make_token(TokenType::Plus),
		Some('-') => make_token(TokenType::Minus),
		Some('*') => make_token(TokenType::Star),
		Some('/') => make_token(TokenType::Slash),
		Some('>') => make_token(TokenType::Greater),
		Some('<') => make_token(TokenType::Less),

		Some('|') => make_token(TokenType::Pipe),
		Some('&') => make_token(TokenType::Ampersand),
		Some('!') => make_token(TokenType::Bang),

		_ => return None
	};

	cursor.consume();

	Some(token)
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

	if !cursor.matches_predicate(starting_predicate) {
		return None;
	}

	let offset = cursor.consumed;
	let mut lexeme = String::new();
	while !cursor.empty() && cursor.matches_predicate(suffix_predicate) {
		lexeme.push(cursor.consume().unwrap());
	}

	if lexeme.len() == 1 && lexeme.starts_with('_') {
		Some(Token::new(TokenType::UnderScore, cursor.consumed - offset))
	} else if let Some(keyword) = match_identifier_to_keyword(&lexeme) {
		Some(Token::new(keyword, cursor.consumed - offset))
	} else {
		Some(Token::new(TokenType::Identifier, cursor.consumed - offset))
	}
}

fn scan_string_literal(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.matches('"') {
		return None;
	}

	let offset = cursor.consumed;
	cursor.consume();

	while !cursor.empty() && cursor.matches_predicate(|c| c != '"') {
		cursor.consume();
	}

	if cursor.matches('"') {
		cursor.consume();
		Some(Token::new(TokenType::StringLiteral, cursor.consumed - offset))
	} else {
		Some(Token::new(TokenType::TombStone, cursor.consumed - offset))
	}
}

fn scan_glyph_literal(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.matches('\'') {
		return None;
	}

	let offset = cursor.consumed;
	cursor.consume();

	while !cursor.empty() && cursor.matches_predicate(|c| c != '\'') {
		cursor.consume();
	}

	if cursor.matches('\'') {
		cursor.consume();
		Some(Token::new(TokenType::GlyphLiteral, cursor.consumed - offset))
	} else {
		Some(Token::new(TokenType::TombStone, cursor.consumed - offset))
	}
}

fn scan_numeric_literal(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.matches_predicate(|c| c.is_digit(10)) {
		return None;
	}

	let offset = cursor.consumed;

	while !cursor.empty() && cursor.matches_predicate(|c| c.is_digit(10)) {
		cursor.consume();
	}

	if cursor.matches('.') {
		cursor.consume();

		while !cursor.empty() && cursor.matches_predicate(|c| c.is_digit(10)) {
			cursor.consume();
		}

		Some(Token::new(TokenType::FloatLiteral, cursor.consumed - offset))
	} else {
		Some(Token::new(TokenType::IntegerLiteral, cursor.consumed - offset))
	}
}

fn lex_two_character_token(cursor: &mut Cursor) -> Option<Token> {
	fn make_token(cursor: &mut Cursor, token_type: TokenType) -> Token {
		cursor.consume();
		cursor.consume();

		Token::new(token_type, 2)
	}

	fn make_token_consume_line(cursor: &mut Cursor, token_type: TokenType) -> Token {
		let mut length = 2;
		cursor.consume();
		cursor.consume();

		while !cursor.empty() && cursor.matches_predicate(|c| c != '\n') {
			length += 1;
			cursor.consume();
		}

		Token::new(token_type, length)
	}

	let current = cursor.current();
	let next = cursor.nth(1);

	if let (Some(current), Some(next)) = (cursor.current(), cursor.nth(1)) {
		let token = match (current, next) {
			('=', '=') => make_token(cursor, TokenType::EqualsEquals),
			('>', '=') => make_token(cursor, TokenType::GreaterEquals),
			('<', '=') => make_token(cursor, TokenType::LessEquals),
			('|', '|') => make_token(cursor, TokenType::PipePipe),
			('&', '&') => make_token(cursor, TokenType::AmpersandAmpersand),
			('/', '/') => make_token_consume_line(cursor, TokenType::CommentLine),
			_ => return None
		};

		Some(token)
	} else {
		None
	}
}

fn match_identifier_to_keyword(lexeme: &str) -> Option<TokenType> {
	match lexeme.to_lowercase().as_str() {
		"fn" => Some(TokenType::Fn),
		"return" => Some(TokenType::Return),
		"if" => Some(TokenType::If),
		"else" => Some(TokenType::Else),
		"for" => Some(TokenType::For),
		"in" => Some(TokenType::In),
		"while" => Some(TokenType::While),
		"let" => Some(TokenType::Let),
		"true" => Some(TokenType::True),
		"false" => Some(TokenType::False),
		_ => None
	}
}

use super::cursor::{Cursor};
use super::syntax::TokenSource;

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
	let mut source = text;
	
	while !source.is_empty() {
		let mut token = next_token(source);
		
		tokens.push(token);
		source = &source[token.length..];
	}
	
	tokens.push(Token::new(TokenType::EndOfFile, 0));
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
		Token::new(TokenType::TombStone, 1)
	}
}


fn lex_whitespace(cursor: &mut Cursor) -> Option<Token> {
	while cursor.current().is_some() && cursor.match_char_predicate(&|c: char| c.is_whitespace()) {
		cursor.next();
	}

	if cursor.len() > 0 {
		Some(Token::new(TokenType::WhiteSpace, cursor.len()))
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
	
	cursor.next();

	Some(token)
}


fn lex_two_character_token(cursor: &mut Cursor) -> Option<Token> {
	fn make_token(cursor: &mut Cursor, token_type: TokenType) -> Token {
		cursor.next();
		cursor.next();

		Token::new(token_type, 2)
	}

	fn make_token_consume_line(cursor: &mut Cursor, token_type: TokenType) -> Token {
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
		Some(Token::new(TokenType::UnderScore, cursor.len()))
	} else if let Some(keyword) = match_identifier_to_keyword(&lexeme) {
		Some(Token::new(keyword, cursor.len()))
	} else {
		Some(Token::new(TokenType::Identifier, cursor.len()))
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
		Some(Token::new(TokenType::StringLiteral, cursor.len()))
	} else {
		Some(Token::new(TokenType::TombStone, cursor.len()))
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
		Some(Token::new(TokenType::GlyphLiteral, cursor.len()))
	} else {
		Some(Token::new(TokenType::TombStone, cursor.len()))
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

		Some(Token::new(TokenType::FloatLiteral, cursor.len()))
	} else {
		Some(Token::new(TokenType::IntegerLiteral, cursor.len()))
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

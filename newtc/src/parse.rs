use std::str::Chars;
use std::fmt::{Display, Formatter};
use std::fmt::Error;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
	Whitespace,
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
	Semicolon,
	Underscore,
	
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
	Tombstone
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

struct Cursor<'a> {
	text: &'a str,
	consumed: usize,
}

impl<'a> Cursor<'a> {
	fn new(text: &'a str) -> Cursor {
		Cursor {
			text,
			consumed: 0,
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
		self.current() == Some(candidate)
	}

	fn matches_predicate<P: Fn(char) -> bool>(&self, predicate: P) -> bool {
		self.current().map(predicate).unwrap_or(false)
	}
	
	fn empty(&self) -> bool {
		self.current().is_none()
	}

	fn chars(&self) -> Chars {
		self.text[self.consumed..].chars()
	}
}

pub fn tokenize(text: &str) -> Vec<Token> {
	let mut tokens: Vec<Token> = vec![];
	let mut cursor = Cursor::new(text);
	
	while cursor.current().is_some() {
		let token = next_token(&mut cursor);
		let preceding_token = tokens.last();
		
		if let Some(preceding_token) = preceding_token {
			match (preceding_token.token_type, token.token_type) {
				(TokenType::Tombstone, TokenType::Tombstone) => {
					let merged_token = Token::merge_as(TokenType::Tombstone, preceding_token, &token);
					tokens.pop();
					tokens.push(merged_token);
				},
				(TokenType::Slash, TokenType::Slash) => {
					let mut comment_line_token = Token::merge_as(TokenType::CommentLine, preceding_token, &token);
					
					while !cursor.empty() && cursor.matches_predicate(|c| c != '\n') {
						cursor.consume();
						comment_line_token.length += 1;
					}
					
					tokens.pop();
					tokens.push(comment_line_token);
				},
				(_, _) => tokens.push(token)
			}
		} else {
			tokens.push(token);
		}
		
		if token.token_type == TokenType::EndOfFile {
			break;
		}
	}
	
	tokens
}

fn merge_adjacent_tokens(token_type: TokenType, preceding_token: Token, tombstone: Token) -> Token {
	Token::new(token_type, preceding_token.length + tombstone.length)
}

fn next_token(cursor: &mut Cursor) -> Token {
	if let Some(token) = lex_whitespace(cursor) {
		token
	} else if cursor.empty() {
		Token::new(TokenType::EndOfFile, 0)
	} else if let Some(token) = lex_multi_character_token(cursor) {
		token
	} else if let Some(token) = lex_two_character_token(cursor) {
		token
	} else if let Some(token) = lex_single_character_token(cursor) {
		token
	} else {
		cursor.consume();
		Token::new(TokenType::Tombstone, 1)
	}
}

fn lex_whitespace(cursor: &mut Cursor) -> Option<Token> {
	let offset = cursor.consumed;
	
	while !cursor.empty() && cursor.matches_predicate(|c| c.is_whitespace()) {
		cursor.consume();
	}
	
	if offset != cursor.consumed {
		Some(Token::new(TokenType::Whitespace, cursor.consumed - offset))
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
		Some(';') => make_token(TokenType::Semicolon),
		Some('_') => make_token(TokenType::Underscore),

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
		None
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
		Some(Token::new(TokenType::Tombstone, cursor.consumed - offset))
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
		Some(Token::new(TokenType::Tombstone, cursor.consumed - offset))
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
/*
There is the concept of a call tree in addition to the usual call stack.
Calling some entry point with the same props would yield the came call tree, 
but for the following additional points:

Each function in the call tree (aka component) can store internal state via hooks.
Each element yielded by the call tree can have event handlers attached to events (e.g. onClick).
These events will either be handled within the call tree (e.g. modifying props or state), do nothing,
or be published to a view model.

View models are bags of properties which are mutated asynchronously -- and as far as newt is 
concerned atomically. These view models can optionally receive certain events.
	
*/

/*
The types:
	primitives: i(8,16,32,64), u(8,16,32,64), f32, f64, glyph, string
	[string is a list of glyphs, a glyph is a unicode glyph, individual bytes are u8]

	complex: struct (and anonymous tuples), enums, tree, list, map
    
The operations:
	match expression
		match expression { option1 => ..., option2 => ..., // must be exhaustive }
	null checks
		no actual null values, optionality is indicated with ? (e.g. int8?) and can only be accessed
		within proper checks. e.g. (let foo: int? = 42; if foo { not null! } else { null... })
*/

/*
Some thoughts:
	no globals, possibly later implement global const expressions (but how powerful can resolver be?)
	avoid excessive syntax -- no one *wants* to learn this, so provide maximal value for minimal cost
	
*/

/*
// start off with simple strong+dynamically typed language
// making the types static later & adding proper annotations + possibly generics
fn main() {
	return (
		<Window height=400 width=400>
			<Span>Hello, World!</Span>
		</Window>
	);
}
*/

/*

type HelloWorldProps struct {
	Lines [string]
}

type HelloWorldEvent enum {
	CounterIncremented,
	Quit // can nest remainder of struct declaration inline
}


fn main() tree { 
	let model HelloWorldProps = useViewModel('MainModel', HelloWorldProps { 
		Lines = []
	});
	let channel fn(HelloWorldEvent) = useChannel<HelloWorldEvent>('MainChannel');
	let children [tree] = useChildren();
	let lines [tree] = [];
	
	for line in model.Lines {
		lines.push((<Span>{line}</Span>));
	}
	
	return (
		<Window height=400 width=400>
			<Button text="more!" onClick={(event ClickEvent) => channel.publish(HelloWorldEvent.CounterIncremented)} />
			<Button text="quit?" onClick={(event ClickEvent) => channel.publish(HelloWorldEvent.Quit)} />
			{lines}
		</Window>
	);
}

*/
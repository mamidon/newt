use std::str::Chars;
use std::fmt::{Display, Formatter};
use std::fmt::Error;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
	Whitespace,
	
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
		
		tokens.push(token);
		
		if token.token_type == TokenType::EndOfFile {
			break;
		}
		
		if token.token_type == TokenType::Tombstone {
			break;
		}
	}
	
	tokens
}

fn next_token(cursor: &mut Cursor) -> Token {
	if let Some(token) = lex_whitespace(cursor) {
		token
	} else if cursor.empty() {
		Token::new(TokenType::EndOfFile, 0)
	} else if let Some(token) = lex_two_character_token(cursor) {
		token
	} else if let Some(token) = lex_single_character_token(cursor) {
		token
	} else if let Some(token) = lex_multi_character_token(cursor) {
		token
	} else {
		Token::new(TokenType::Tombstone, 0)
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
		
		Some('=') => make_token(TokenType::Equals),
		Some('+') => make_token(TokenType::Plus),
		Some('-') => make_token(TokenType::Minus),
		Some('*') => make_token(TokenType::Star),
		Some('/') => make_token(TokenType::Slash),
		Some('>') => make_token(TokenType::Greater),
		Some('<') => make_token(TokenType::Less),
		
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
	} /*else if let Some(glyph) = scan_glyph_literal(cursor) {
		Some(glyph)
	} */else if let Some(identifier) = scan_identifier(cursor) {
		Some(identifier)
	} else {
		None
	}
}

fn scan_identifier(cursor: &mut Cursor) -> Option<Token> {
	if !cursor.matches_predicate(|c| c.is_alphabetic()) {
		return None;
	}
	
	let offset = cursor.consumed;
	while !cursor.empty() && cursor.matches_predicate(|c| c.is_alphanumeric()) {
		cursor.consume();
	}
	
	Some(Token::new(TokenType::Identifier, cursor.consumed - offset))
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
	fn make_token(token_type: TokenType) -> Token {
		Token::new(token_type, 2)
	}

	let current = cursor.current();
	let next = cursor.nth(1);

	if let (Some(current), Some(next)) = (cursor.current(), cursor.nth(1)) {
		let token = match (current, next) {
			('=', '=') => make_token(TokenType::EqualsEquals),
			('>', '=') => make_token(TokenType::GreaterEquals),
			('<', '=') => make_token(TokenType::LessEquals),
			_ => return None
		};
		
		cursor.consume();
		cursor.consume();
		Some(token)
	} else {
		None
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
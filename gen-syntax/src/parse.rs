use crate::tokens::{Token, TokenKind};

struct Tokens<'a> {
	source: &'a [Token],
	tokens_consumed: usize,
}

impl<'a> Tokens<'a> {
	pub fn new(source: &[Token]) -> Tokens {
		Tokens {
			source,
			tokens_consumed: 0
		}
	}

	pub fn peek(&self) -> Token {
		self.peek_nth(0)
	}

	pub fn peek_nth(&self, offset: usize) -> Token {
		if offset < self.source.len() {
			*self.source.iter().nth(offset).unwrap()
		} else {
			Token::new(TokenKind::EndOfFile, 0, 0)
		}
	}

	pub fn tokens_consumed(&self) -> usize {
		self.tokens_consumed
	}

	pub fn consume(&mut self) {
		if self.source.len() >= 1 {
			self.tokens_consumed += 1;
			self.source = &self.source[1..];
		}
	}

	pub fn expect(&mut self, expected: TokenKind) -> Result<Token, GrammarParseError> {
		let current = self.peek();
		self.consume();

		if current.kind == expected {
			Ok(current)
		} else {
			Err(GrammarParseError {
				expected,
				actual: current
			})
		}
	}
}

pub fn parse(tokens: Vec<Token>) -> Vec<GrammarSyntax> {
	let non_trivia: Vec<Token> = tokens.iter()
		.filter(|t| !t.is_trivia())
		.map(|t| *t)
		.collect();

	let mut cursor = Tokens::new(non_trivia.as_slice());

	root(&mut cursor)
}

fn root(cursor: &mut Tokens) -> Vec<GrammarSyntax> {
	println!("root");

	let mut rules: Vec<GrammarSyntax> = vec![];

	while cursor.peek().kind != TokenKind::EndOfFile {

		rules.push(rule(cursor).unwrap_or_else(|e| GrammarSyntax::Error(e)));
	}

	rules
}

fn rule(cursor: &mut Tokens) -> Result<GrammarSyntax, GrammarParseError> {
	println!("rule");
	let name = cursor.expect(TokenKind::Identifier)?;
	cursor.expect(TokenKind::Arrow)?;
	let sequence = production_sequence(cursor)?;
	cursor.expect(TokenKind::SemiColon)?;

	Ok(GrammarSyntax::Rule(GrammarRule {
		name,
		production: sequence
	}))
}

fn production_sequence(cursor: &mut Tokens) -> Result<GrammarProduction, GrammarParseError> {
	println!("sequence");
	let mut sequence: Vec<GrammarProduction> = vec![];

	loop {
		let token = cursor.peek();
		let next = match token.kind {
			TokenKind::Identifier => production_identifier(cursor)?,
			TokenKind::Plus | TokenKind::Star => production_operator(cursor, sequence.pop())?,
			TokenKind::LeftParenthesis => production_grouping(cursor)?,
			TokenKind::Pipe => production_pipe(cursor, sequence.pop())?,
			_ => break
		};

		sequence.push(next);
	}

	Ok(GrammarProduction::Sequence(sequence.into_boxed_slice()))
}

fn production_identifier(cursor: &mut Tokens) -> Result<GrammarProduction, GrammarParseError> {
	println!("identifier");
	let rule_name = cursor.expect(TokenKind::Identifier)?;
	let member_name = if cursor.peek().kind == TokenKind::LeftBracket {
		cursor.expect(TokenKind::LeftBracket)?;
		let name = cursor.expect(TokenKind::Identifier)?;
		cursor.expect(TokenKind::RightBracket)?;
		Some(name)
	} else {
		None
	};

	Ok(GrammarProduction::Identifier(GrammarIdentifier { rule_name, member_name }))
}

fn production_operator(cursor: &mut Tokens, lhs: Option<GrammarProduction>) -> Result<GrammarProduction, GrammarParseError> {
	println!("operator");
	let lhs = lhs.ok_or(GrammarParseError { expected: TokenKind::Identifier, actual: Token::new(TokenKind::Error, 0, 0)})?;
	let lhs = Box::new(lhs);
	let token = cursor.peek().kind;
	cursor.consume();

	match token {
		TokenKind::Plus => Ok(GrammarProduction::Plus(lhs)),
		TokenKind::Star => Ok(GrammarProduction::Star(lhs)),
		_ => panic!("production_sequence should've only sent production_operator a + or *")
	}
}

fn production_grouping(cursor: &mut Tokens) -> Result<GrammarProduction, GrammarParseError> {
	println!("grouping");
	cursor.expect(TokenKind::LeftParenthesis)?;
	let sequence = production_sequence(cursor)?;
	cursor.expect(TokenKind::RightParenthesis)?;

	Ok(GrammarProduction::Grouping(Box::new(sequence)))
}

fn production_pipe(cursor: &mut Tokens, lhs: Option<GrammarProduction>) -> Result<GrammarProduction, GrammarParseError> {
	println!("pipe");
	let lhs = lhs.ok_or(GrammarParseError { expected: TokenKind::Identifier, actual: Token::new(TokenKind::Error, 0, 0)})?;
	cursor.expect(TokenKind::Pipe)?;
	let sequence = production_sequence(cursor)?;

	Ok(GrammarProduction::Pipe(Box::new(lhs), Box::new(sequence)))
}

#[derive(Debug)]
pub struct GrammarParseError {
	expected: TokenKind,
	actual: Token
}

#[derive(Debug)]
pub struct GrammarRule {
	name: Token,
	production: GrammarProduction
}

#[derive(Debug)]
pub struct GrammarIdentifier {
	rule_name: Token,
	member_name: Option<Token>
}

#[derive(Debug)]
pub enum GrammarSyntax {
	Error(GrammarParseError),

	Rule(GrammarRule),
	Production(GrammarProduction)
}

#[derive(Debug)]
pub enum GrammarProduction {
	Plus(Box<GrammarProduction>),
	Star(Box<GrammarProduction>),
	Sequence(Box<[GrammarProduction]>),
	Grouping(Box<GrammarProduction>),
	Pipe(Box<GrammarProduction>, Box<GrammarProduction>),
	Identifier(GrammarIdentifier)
}

/*

Expr => UnaryExpr+ | BinaryExpr*
BinaryExpr => Expr[lhs] Token[op] Expr[rhs]
*/

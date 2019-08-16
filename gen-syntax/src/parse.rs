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

	pub fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
		let actual = self.peek();
		self.consume();

		if actual.kind == expected {
			Ok(actual)
		} else {
			Err(ParseError::unexpected(expected, actual))
		}
	}
}

pub fn parse(tokens: Vec<Token>) -> Vec<Syntax> {
	let non_trivia: Vec<Token> = tokens.iter()
		.filter(|t| !t.is_trivia())
		.map(|t| *t)
		.collect();

	let mut cursor = Tokens::new(non_trivia.as_slice());

	root(&mut cursor)
}

fn root(cursor: &mut Tokens) -> Vec<Syntax> {
	let mut rules: Vec<Syntax> = vec![];

	while cursor.peek().kind != TokenKind::EndOfFile {

		rules.push(rule(cursor).unwrap_or_else(|e| Syntax::Error(e)));
	}

	rules
}

fn rule(cursor: &mut Tokens) -> Result<Syntax, ParseError> {
	let name = cursor.expect(TokenKind::Identifier)?;
	cursor.expect(TokenKind::Arrow)?;
	let sequence = production_sequence(cursor)?;
	cursor.expect(TokenKind::SemiColon)?;

	Ok(Syntax::Rule(Grammar {
		name,
		production: sequence
	}))
}

fn production_sequence(cursor: &mut Tokens) -> Result<Production, ParseError> {
	let mut sequence: Vec<Production> = vec![];

	loop {
		let token = cursor.peek();
		let next = match token.kind {
			TokenKind::Identifier => production_identifier(cursor)?,
			TokenKind::Plus | TokenKind::Star => production_operator(cursor, sequence.pop())?,
			TokenKind::LeftParenthesis => production_grouping(cursor)?,
			TokenKind::Pipe => production_pipe(cursor, sequence.pop())?,
			TokenKind::Quoted => {
				/* Not much to do for this right now */
				cursor.consume();
				continue;
			},
			_ => break
		};

		sequence.push(next);
	}

	if sequence.len() != 1 {
		Ok(Production::Sequence(sequence.into_boxed_slice()))
	} else {
		Ok(sequence.pop().unwrap())
	}
}

fn production_identifier(cursor: &mut Tokens) -> Result<Production, ParseError> {
	let rule_name = cursor.expect(TokenKind::Identifier)?;
	let member_name = if cursor.peek().kind == TokenKind::LeftBracket {
		cursor.expect(TokenKind::LeftBracket)?;
		let name = cursor.expect(TokenKind::Identifier)?;
		cursor.expect(TokenKind::RightBracket)?;
		Some(name)
	} else {
		None
	};

	Ok(Production::Identifier(GrammarIdentifier { rule_name, member_name }))
}

fn production_operator(cursor: &mut Tokens, lhs: Option<Production>) -> Result<Production, ParseError> {
	let lhs = lhs.ok_or(ParseError::missing("No production sequence for operator"))?;
	let lhs = Box::new(lhs);
	let token = cursor.peek().kind;
	cursor.consume();

	match token {
		TokenKind::Plus => Ok(Production::Plus(lhs)),
		TokenKind::Star => Ok(Production::Star(lhs)),
		_ => panic!("production_sequence should've only sent production_operator a + or *")
	}
}

fn production_grouping(cursor: &mut Tokens) -> Result<Production, ParseError> {
	cursor.expect(TokenKind::LeftParenthesis)?;
	let sequence = production_sequence(cursor)?;
	cursor.expect(TokenKind::RightParenthesis)?;

	Ok(Production::Grouping(Box::new(sequence)))
}

fn production_pipe(cursor: &mut Tokens, lhs: Option<Production>) -> Result<Production, ParseError> {
	let lhs = lhs.ok_or(ParseError::missing("No production sequence found"))?;
	cursor.expect(TokenKind::Pipe)?;
	let sequence = production_sequence(cursor)?;

	Ok(Production::Pipe(Box::new(lhs), Box::new(sequence)))
}

#[derive(Debug)]
pub enum ParseError {
	UnexpectedToken { expected: TokenKind, actual: Token },
	MissingSyntax { message: &'static str }
}

impl ParseError {
	pub fn unexpected(expected: TokenKind, actual: Token) -> ParseError {
		ParseError::UnexpectedToken { expected, actual }
	}

	pub fn missing(message: &'static str) -> ParseError {
		ParseError::MissingSyntax { message }
	}
}

#[derive(Debug)]
pub struct Grammar {
	name: Token,
	production: Production
}

#[derive(Debug)]
pub struct GrammarIdentifier {
	rule_name: Token,
	member_name: Option<Token>
}

#[derive(Debug)]
pub enum Syntax {
	Error(ParseError),
	Rule(Grammar),
}

#[derive(Debug)]
pub enum Production {
	Plus(Box<Production>),
	Star(Box<Production>),
	Sequence(Box<[Production]>),
	Grouping(Box<Production>),
	Pipe(Box<Production>, Box<Production>),
	Identifier(GrammarIdentifier)
}

/*

Expr => UnaryExpr+ | BinaryExpr*
BinaryExpr => Expr[lhs] Token[op] Expr[rhs]
*/

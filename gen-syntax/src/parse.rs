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
			Err(ParseError::new(actual, ParseErrorKind::UnexpectedToken {
				expected,
				actual: actual.kind
			}))
		}
	}
}


#[derive(Debug)]
pub struct ParseError {
	pub(crate) location: Token,
	pub(crate) kind: ParseErrorKind
}

#[derive(Debug)]
pub enum ParseErrorKind {
	UnexpectedToken { expected: TokenKind, actual: TokenKind },
	MissingSyntax { message: &'static str }
}

impl ParseError {
	pub fn new(location: Token, kind: ParseErrorKind) -> ParseError {
		ParseError {
			kind,
			location
		}
	}
}

#[derive(Debug)]
pub enum Production {
	Error(ParseError),
	Root(Box<[Production]>),
	Rule { name: Token, production: Box<Production> },
	Plus(Box<Production>),
	Star(Box<Production>),
	Grouping(Box<Production>),
	Sequence(Box<[Production]>),
	Pipe(Box<[Production]>),
	Identifier { rule_name: Token, member_name: Option<Token> }
}

pub struct ProductionIterator<'a> {
	frontier: Vec<&'a Production>
}

impl Production {
	pub fn iter(&self) -> ProductionIterator {
		ProductionIterator::from(self)
	}
}

impl<'a> From<&'a Production> for ProductionIterator<'a> {
	fn from(root: &'a Production) -> Self {
		ProductionIterator {
			frontier: vec![root]
		}
	}
}

impl<'a> Iterator for ProductionIterator<'a> {
	type Item = &'a Production;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.frontier.pop()?;

		match next {
			Production::Pipe(chain) => {
				self.frontier.extend(chain.iter().rev());
			},
			Production::Sequence(sequence) => {
				self.frontier.extend(sequence.iter().rev())
			},
			Production::Grouping(child)
			| Production::Plus(child)
			| Production::Star(child) => {
				self.frontier.push(child)
			},
			Production::Rule { name: _, production } => {
				self.frontier.push(&production)
			},
			Production::Error(_) => {},
			Production::Root(items) => {
				self.frontier.extend(items.iter().rev())
			},
			Production::Identifier { rule_name: _, member_name: _ } => {},
		};

		Some(next)
	}
}

impl<'a> IntoIterator for &'a Production {
	type Item = &'a Production;
	type IntoIter = ProductionIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ProductionIterator::from(self)
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Production, Vec<ParseError>> {
	let non_trivia: Vec<Token> = tokens.iter()
		.filter(|t| !t.is_trivia())
		.map(|t| *t)
		.collect();

	let mut cursor = Tokens::new(non_trivia.as_slice());

	root(&mut cursor)
}

fn root(cursor: &mut Tokens) -> Result<Production, Vec<ParseError>> {
	let mut rules: Vec<Production> = vec![];
	let mut errors: Vec<ParseError> = vec![];

	while cursor.peek().kind != TokenKind::EndOfFile {
		match rule(cursor) {
			Ok(p) => rules.push(p),
			Err(e) => {
				errors.push(e);
				while cursor.peek().kind != TokenKind::SemiColon && cursor.peek().kind != TokenKind::EndOfFile {
					cursor.consume();
				}
				if cursor.peek().kind == TokenKind::SemiColon {
					cursor.expect(TokenKind::SemiColon).unwrap();
				}
			}
		};
	}

	if errors.len() == 0 {
		Ok(Production::Root(rules.into_boxed_slice()))
	} else {
		Err(errors)
	}
}

fn rule(cursor: &mut Tokens) -> Result<Production, ParseError> {
	let name = cursor.expect(TokenKind::Identifier)?;
	cursor.expect(TokenKind::Arrow)?;
	let sequence = production_pipe(cursor)?;
	cursor.expect(TokenKind::SemiColon)?;

	Ok(Production::Rule {
		name,
		production: Box::new(sequence)
	})
}

fn production_sequence(cursor: &mut Tokens) -> Result<Production, ParseError> {
	let mut sequence: Vec<Production> = vec![];

	loop {
		let token = cursor.peek();
		let next = match token.kind {
			TokenKind::Identifier => production_identifier(cursor)?,
			TokenKind::Plus | TokenKind::Star => production_operator(cursor, sequence.pop())?,
			TokenKind::LeftParenthesis => production_grouping(cursor)?,
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

	Ok(Production::Identifier { rule_name, member_name })
}

fn production_operator(cursor: &mut Tokens, lhs: Option<Production>) -> Result<Production, ParseError> {
	let lhs = lhs.ok_or(ParseError::new(cursor.peek(), ParseErrorKind::MissingSyntax { message: "No production sequence for operator" }))?;
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
	let sequence = production_pipe(cursor)?;
	cursor.expect(TokenKind::RightParenthesis)?;

	Ok(Production::Grouping(Box::new(sequence)))
}

fn production_pipe(cursor: &mut Tokens) -> Result<Production, ParseError> {
	let lhs = production_sequence(cursor)?;
	let mut chain = vec![lhs];

	while cursor.peek().kind == TokenKind::Pipe {
		cursor.consume();
		chain.push(production_sequence(cursor)?);
	}

	if chain.len() != 1 {
		Ok(Production::Pipe(chain.into_boxed_slice()))
	} else {
		Ok(chain.pop().unwrap())
	}
}
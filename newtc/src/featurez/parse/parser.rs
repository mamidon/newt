use super::*;

pub struct Parser<'a> {
	text: &'a str,
	source: StrTokenSource,
	consumed_tokens: usize,
	events: Vec<ParseEvent>,
	errors: Vec<&'static str>,
}

impl<'a> Parser<'a> {
	pub fn new(text: &'a str, source: StrTokenSource) -> Parser<'a> {
		Parser {
			text,
			source,
			consumed_tokens: 0,
			events: vec![],
			errors: vec![],
		}
	}

	pub fn current(&self) -> TokenKind {
		self.source.token(self.consumed_tokens).token_kind()
	}

	pub fn current2(&self) -> Option<(TokenKind, TokenKind)> {
		self.source.token2(self.consumed_tokens)
			.map(|(t1,t2)| (t1.token_kind(), t2.token_kind()))
	}

	pub fn nth(&self, n: usize) -> TokenKind {
		self.source.token(self.consumed_tokens + n).token_kind()
	}

	pub fn match_token_kind(&self, kind: TokenKind) -> bool {
		self.source.token_kind(self.consumed_tokens) == kind
	}

	pub fn token(&mut self, kind: TokenKind) {
		self.consumed_tokens += 1;
		self.events.push(ParseEvent::Token { kind });

		self.eat_trivia();
	}

	pub fn begin_node(&mut self) -> Marker {
		let index = self.events.len();
		self.events.push(ParseEvent::tombstone());

		Marker::new(index)
	}

	pub fn end_node(&mut self, marker: &mut Marker, kind: SyntaxKind) {
		let begin = &mut self.events[marker.index()];

		match begin {
			ParseEvent::BeginNode { kind: ref mut slot } => {
				marker.disable();
				*slot = kind;
			}
			_ => panic!("Did not expect to complete a marker we don't have access to anymore!"),
		};

		self.events.push(ParseEvent::EndNode);
	}

	fn eat_trivia(&mut self) {
		loop {
			match self.current() {
				TokenKind::WhiteSpace
				| TokenKind::TombStone
				| TokenKind::CommentLine
				| TokenKind::CommentBlock => {}
				_ => break,
			}

			self.consumed_tokens += 1;
		}
	}
}


impl<'a> Display for Parser<'a> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		use std::iter::repeat;

		let mut depth = 0;
		let mut prefix: String = repeat("\t").take(depth).collect();

		for event in &self.events {
			match event {
				ParseEvent::BeginNode { kind: SyntaxKind::TombStone } => {},
				ParseEvent::BeginNode { kind: _ } => {
					writeln!(f, "{}{:?}", prefix, event);

					depth += 1;
					prefix = repeat("\t").take(depth).collect();
				}
				ParseEvent::EndNode => {
					depth -= 1;
					prefix = repeat("\t").take(depth).collect();

					writeln!(f, "{}{:?}", prefix, event);
				}
				_ => {
					writeln!(f, "{}{:?}", prefix, event);
				}
			}
		}
		Ok(())
	}
}
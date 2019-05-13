use crate::featurez::syntax::{SyntaxKind, TokenSource};
use crate::featurez::tokens::TokenKind;
use crate::featurez::parse::ParseEvent;
use crate::featurez::StrTokenSource;
use crate::featurez::parse::marker::Marker;

pub struct Parser {
	source: StrTokenSource,
	consumed_tokens: usize,
	events: Vec<ParseEvent>,
	panicking: bool,
}

impl Parser {
	pub fn new(source: StrTokenSource) -> Parser {
		let mut p = Parser {
			source,
			consumed_tokens: 0,
			events: vec![],
			panicking: false,
		};
		p.eat_trivia();
		
		p
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

	pub fn token_if(&mut self, kind: TokenKind) -> bool {
		if self.current() != kind {
			return false;
		}

		let token = self.source.token(self.consumed_tokens);
		self.consumed_tokens += 1;
		self.events.push(ParseEvent::Token { kind: token.token_kind(), length: token.lexeme_length() });

		self.eat_trivia();
		
		return true;
	}
	
	pub fn expect_token_kind(&mut self, kind: TokenKind, msg: &'static str) {
		if self.token_if(kind) {
			self.panicking = false;
			return;
		}
		
		if self.panicking {
			self.remap_token(TokenKind::TombStone);
		} else {
			self.panicking = true;
			
			self.report_error(msg);
		}
	}
	
	fn report_error(&mut self, message: &'static str) {
		let mut error = self.begin_node();

		let token = self.source.token(self.consumed_tokens);

		self.consumed_tokens += 1;
		self.events.push(ParseEvent::Token { kind: token.token_kind(), length: token.lexeme_length() });

		self.eat_trivia();
		
		self.end_node(&mut error, SyntaxKind::Error(message));
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
	
	pub fn end_parsing(mut self) -> Vec<ParseEvent> {
		self.eat_remaining_tokens();
		
		self.events.into_iter().filter(|e| match e { 
			ParseEvent::BeginNode { kind: SyntaxKind::TombStone } => false,
			_ => true
		}).collect()
	}
	
	fn remap_token(&mut self, kind: TokenKind) {
		let token = self.source.token(self.consumed_tokens);

		self.consumed_tokens += 1;
		self.events.push(ParseEvent::Token { kind, length: token.lexeme_length() });

		self.eat_trivia();
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

			let token = self.source.token(self.consumed_tokens);
			self.events.push(ParseEvent::Trivia { kind: token.token_kind(), length: token.lexeme_length() });
			self.consumed_tokens += 1;
		}
	}
	
	fn eat_remaining_tokens(&mut self) {
		if self.current() == TokenKind::EndOfFile {
			return;
		}

		self.events.pop(); // crack open the root element
		
		let mut remaining = self.begin_node();

		loop {
			if self.current() == TokenKind::EndOfFile {
				break;
			}
			
			
			let token = self.source.token(self.consumed_tokens);
			self.events.push(ParseEvent::Trivia { kind: token.token_kind(), length: token.lexeme_length() });
			self.consumed_tokens += 1;
		}

		self.end_node(&mut remaining, SyntaxKind::Error("Unexpected text"));
		
		self.events.push(ParseEvent::EndNode); // close the root element
	}
}

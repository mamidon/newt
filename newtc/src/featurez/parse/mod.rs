use crate::featurez::tokens::{
	TokenKind,
	Token,
	tokenize,
	StrTokenSource
};
use crate::featurez::syntax::{
	TokenSource,
	SyntaxElement,
	SyntaxToken,
	SyntaxKind,
	SyntaxNode,
	SyntaxTree,
	TextTreeSink,
	TreeSink,
	LiteralExprNode,
	BinaryExprNode
};
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub enum ParseEvent {
	Token {
		token: Token
	},
	BeginNode {
		kind: SyntaxKind
	},
	EndNode,
}

impl ParseEvent {
	pub fn tombstone() -> ParseEvent {
		ParseEvent::BeginNode { kind: SyntaxKind::TombStone }
	}
}

pub struct Marker {
	index: usize,
	disabled: bool
}

impl Marker {
	pub fn new(index: usize) -> Marker {
		Marker {
			index,
			disabled: false
		}
	}
	
	pub fn disable(&mut self) {
		self.disabled = true;
	}
	
	pub fn abandon(&mut self) {
		self.disabled = true;
	}
}

impl Drop for Marker {
	fn drop(&mut self) {
		if !self.disabled {
			panic!("You must disable or abandon the marker!")
		}
	}
}



pub struct Parser<'a> {
	text: &'a str,
	source: StrTokenSource,
	consumed_tokens: usize,
	events: Vec<ParseEvent>,
	errors: Vec<&'static str>
}

impl<'a> Display for Parser<'a> {
	
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		use std::iter::repeat;

		let mut depth = 0;
		let mut prefix: String = repeat("\t").take(depth).collect();
		
		for event in &self.events {
			match event {
				ParseEvent::BeginNode { kind: _ } => {
					writeln!(f, "{}{:?}", prefix, event);
					
					depth += 1;
					prefix = repeat("\t").take(depth).collect();
				},
				ParseEvent::EndNode => {
					depth -= 1;
					prefix = repeat("\t").take(depth).collect();
					
					writeln!(f, "{}{:?}", prefix, event);
				},
				_ => {
					writeln!(f, "{}{:?}", prefix, event);
				}
			}
		}
		Ok(())
	}
}

impl<'a> Parser<'a> {
	pub fn new(text: &'a str, source: StrTokenSource) -> Parser<'a> {
		Parser {
			text,
			source,
			consumed_tokens: 0,
			events: vec![],
			errors: vec![]
		}
	}
	
	pub fn current(&self) -> Token {
		self.source.token(self.consumed_tokens)
	}
	
	pub fn current2(&self) -> Option<(Token, Token)> {
		self.source.token2(self.consumed_tokens)
	}
	
	pub fn nth(&self, n: usize) -> Token {
		self.source.token(self.consumed_tokens + n)
	}
	
	pub fn match_token_kind(&self, kind: TokenKind) -> bool {
		self.source.token_kind(self.consumed_tokens) == kind
	}
	
	pub fn token(&mut self, token: Token) {
		self.consumed_tokens += 1;
		self.events.push(ParseEvent::Token { token });
		
		self.eat_trivia();
	}
	
	pub fn begin_node(&mut self) -> Marker {
		let index = self.events.len();
		self.events.push(ParseEvent::tombstone());
		
		Marker::new(index)
	}
	
	pub fn end_node(&mut self, marker: &mut Marker, kind: SyntaxKind) {
		let begin = &mut self.events[marker.index];
		
		match begin {
			ParseEvent::BeginNode { kind: ref mut slot } => {
				marker.disable();
				*slot = kind;
			},
			_ => panic!("Did not expect to complete a marker we don't have access to anymore!")
		};
		
		self.events.push(ParseEvent::EndNode);
	}
	
	fn eat_trivia(&mut self) {
		loop {
			match self.current().token_kind() {
				TokenKind::WhiteSpace
				| TokenKind::TombStone 
				| TokenKind::CommentLine 
				| TokenKind::CommentBlock => {},
				_ => break
			}
			
			self.consumed_tokens += 1;
		}
	}
}

pub fn parse<F: FnOnce(&mut Parser) -> ()>(text: &str, root: F) {
	let tokens = tokenize(text);
	let source = StrTokenSource::new(tokens);
	let mut parser = Parser::new(text, source);
	
	root(&mut parser);
	println!("{}", parser);
}

// events lifecycle for 1+2+3

// |1+2+3
// [(start)]

// |1+2+3
// [(start), (start)]

// 1|+2+3
// [(start), (start), int_lit_token ]

// 1+|2+3
// [(start), (start), int_lit_token, plus_token ]

// 1+|2+3
// [(start), (start), int_lit_token, plus_token, (start) ]


// 1+2|+3
// [(start), (start), int_lit_token, plus_token, (start), int_lit_token]

// 1+2+|3
// [(start), (start), int_lit_token, plus_token, 
// 	(start), int_lit_token, plus_token ]

// 1+2+3|
// [(start), (start), int_lit_token, plus_token, 
// 	(start), int_lit_token, plus_token, int_lit_token ]


// 1+2+3|
// [(start_expr), (start_add), int_lit_token, plus_token, 
// 	(start_add), int_lit_token, plus_token, int_lit_token (finish) (finish) (finish) ]

// expr
// |-add
// |--int_lit_token
// |--plus_token
// |--add
// |---int_lit_token
// |---plus_token
// |---int_lit_token
//

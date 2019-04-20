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

enum ParseEvent {
	BeginNode {
		kind: SyntaxKind
	},
	EndNode,
}

struct Parser<'a> {
	text: &'a str,
	source: StrTokenSource,
	consumed_tokens: usize,
	consumed_text: usize,
	events: Vec<ParseEvent>,
	errors: Vec<&'static str>
}

impl<'a> Parser<'a> {
	pub fn new(text: &'a str, source: StrTokenSource) -> Parser<'a> {
		Parser {
			text,
			source,
			consumed_tokens: 0,
			consumed_text: 0,
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
}

mod grammar {
	use crate::featurez::parse::Parser;
	use self::expr::*;
	
	mod expr {
		use crate::featurez::parse::Parser;
		use crate::featurez::syntax::{SyntaxNode, SyntaxKind};
		use crate::featurez::{Token, TokenKind};

		pub fn expr(p: &mut Parser) {
			add_expr(p);
		}
		
		pub fn add_expr(p: &mut Parser) {
			let left = integer_literal_expr(p);
			
			unimplemented!();
		}
		
		pub fn integer_literal_expr(p: &mut Parser) -> Option<SyntaxNode> {
			let current = p.current();
			
			unimplemented!();
		}
	}
	
	pub fn root(p: &mut Parser) {
		expr(p);
	}
}

pub fn parse<F: FnOnce(&Parser) -> ()>(text: &str, root: F) {
	let tokens = tokenize(text);
	let source = StrTokenSource::new(tokens);
	let parser = Parser::new(text, source);
	
	root(&parser);
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

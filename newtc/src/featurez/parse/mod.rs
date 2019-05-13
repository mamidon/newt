use crate::featurez::syntax::{SyntaxTree, SyntaxElement};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};
use crate::featurez::runtime::{ExprVirtualMachine, ExprVisitor};
use crate::featurez::syntax::{ExprNode, AstNode};
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;


mod parse_event;
pub use self::parse_event::ParseEvent;

mod marker;
use self::marker::Marker;

mod parser;
pub use self::parser::Parser;
use std::fmt::Debug;

pub fn parse(text: &str) -> SyntaxTree {
	use super::grammar::root;
	
    let tokens = tokenize(text);
    let source = StrTokenSource::new(tokens);
    let mut parser = Parser::new(source);

    root(&mut parser);
	
	let tree = SyntaxTree::from_parser(parser, text);
	let machine = ExprVirtualMachine::new();
	
	match tree.root() {
		SyntaxElement::Node(n) => {
			let root = ExprNode::cast(&n).unwrap();
			let result: i32 = machine.visit_expr(root);
			println!("RESULT: {}", result);
		},
		_ => unimplemented!()
	};
	
	// implement test coverage for (str)TokenSource, Parser, SyntaxTree
	tree
}

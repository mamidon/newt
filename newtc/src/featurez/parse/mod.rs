use crate::featurez::syntax::{
	BinaryExprNode, LiteralExprNode, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
	SyntaxTree, TreeSink, TokenSource, TreeSink,
};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

mod parse_event;
pub use self::parse_event::ParseEvent;

mod marker;
use self::marker::Marker;

mod parser;
pub use self::parser::Parser;

pub fn parse(text: &str) -> SyntaxTree {
	use super::grammar::root;
	
    let tokens = tokenize(text);
    let source = StrTokenSource::new(tokens);
    let mut parser = Parser::new(text, source);

    root(&mut parser);
	
	let tree = SyntaxTree::from_parser(parser, text);
	
	tree
}


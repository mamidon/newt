#![cfg(test)]

use super::*;

use insta::assert_debug_snapshot_matches;

use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::TokenKind;
use crate::featurez::Parser;
use crate::featurez::grammar::root;

#[test]
fn left_associativity_is_deeply_nested() {
    let text = "1+2+3";
    let tokens = tokenize(text);
    let source = StrTokenSource::new(tokens);
	let mut parser = Parser::new(source);

	root(&mut parser);
	
    let tree = SyntaxTree::from_parser(parser, text);
	
	assert_debug_snapshot_matches!("left_associativity_is_deeply_nested", tree);
}


#[test]
fn right_associativity_is_deeply_nested() {
	let text = "1+2*3";
	let tokens = tokenize(text);
	let source = StrTokenSource::new(tokens);
	let mut parser = Parser::new(source);

	root(&mut parser);

	let tree = SyntaxTree::from_parser(parser, text);

	assert_debug_snapshot_matches!("right_associativity_is_deeply_nested", tree);
}
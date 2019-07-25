#![cfg(test)]

use super::*;

use insta::assert_snapshot_matches;

use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::TokenKind;
use crate::featurez::Parser;
use crate::featurez::grammar::root;

macro_rules! syntax_tree_tests {
	($($name:ident: $test_source:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let text: &str = $test_source;
			let tokens = tokenize(text);
			let source = StrTokenSource::new(tokens);
			let mut parser = Parser::new(source);
			
			root(&mut parser);
			
			let tree = SyntaxTree::from_parser(parser, text);
			let approval_document = format!("{}\n===>\n{:#?}", text, tree);
			assert_snapshot_matches!(stringify!($name), approval_document);
		}
	)*
	}
}

syntax_tree_tests! {
	left_associativity_is_deeply_nested: "1+2+3",
	higher_precedence_is_evaluated_first: "1+2*3",
	higher_precedence_is_noop_when_first: "1*2+3",
	unary_operators_are_parsed: "-1*2+-3",
}

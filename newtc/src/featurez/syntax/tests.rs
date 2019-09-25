#![cfg(test)]

use super::*;

use insta::assert_snapshot_matches;

use crate::featurez::grammar::{root_expr, root_stmt};
use crate::featurez::parse::Parser;
use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::TokenKind;

macro_rules! syntax_tree_expr_tests {
	($($name:ident: $test_source:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let text: &str = $test_source;
			let tokens = tokenize(text);
			let parser = Parser::new(StrTokenSource::new(tokens.clone()));
			let completed_parsing = root_expr(parser);
			let tree = SyntaxTree::from_parser(&completed_parsing, text);

			let approval_document = format!("====text====\n============\n{}\n\
				====tokens====\n============\n{:#?}\n\
				====events====\n============\n{:#?}\n\
				====tree====\n============\n{:#?}", text, tokens, completed_parsing.events, tree);
	assert_snapshot_matches!(stringify!($name), approval_document);
		}
	)*
	}
}

macro_rules! syntax_tree_stmt_tests {
	($($name:ident: $test_source:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let text: &str = $test_source;
			let tokens = tokenize(text);
			let mut parser = Parser::new(StrTokenSource::new(tokens.clone()));
			let completed_parsing = root_stmt(parser);
			let tree = SyntaxTree::from_parser(&completed_parsing, text);

			let approval_document = format!("====text====\n============\n{}\n\
				====tokens====\n============\n{:#?}\n\
				====events====\n============\n{:#?}\n\
				====tree====\n============\n{:#?}", text, tokens, completed_parsing.events, tree);
	assert_snapshot_matches!(stringify!($name), approval_document);
		}
	)*
	}
}

// associativity & precedence
syntax_tree_expr_tests! {
    left_associativity_is_deeply_nested: "1+2+3",
    higher_precedence_is_evaluated_first: "1+2*3",
    higher_precedence_is_noop_when_first: "1*2+3",
    unary_operators_are_properly_grouped: "-1*2+-3",
    nested_unary_operators: "1*--2.12",
    grouping_is_highest_precedence: "(1+2)*3",
    expr_starting_whitespace_is_fine: r#"
    1*2"#,
}

syntax_tree_stmt_tests! {
    stmt_starting_whitespace_is_fine: r#"
	let x = 1;
	let y = 2;
	let c = x * 2 + y;
	"#,
}

// operators

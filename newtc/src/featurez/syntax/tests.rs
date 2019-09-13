#![cfg(test)]

use super::*;

use insta::assert_snapshot_matches;

use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::TokenKind;
use crate::featurez::parse::Parser;
use crate::featurez::grammar::{root_expr, root};

macro_rules! syntax_tree_expr_tests {
	($($name:ident: $test_source:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let text: &str = $test_source;
			let tokens = tokenize(text);
			let source = StrTokenSource::new(tokens);
			let mut parser = Parser::new(source);
			
			root_expr(&mut parser);
			
			let tree = SyntaxTree::from_parser(parser, text);
			let approval_document = format!("{}\n===>\n{:#?}", text, tree);
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
			let source = StrTokenSource::new(tokens);
			let mut parser = Parser::new(source);

			root_expr(&mut parser);

			let tree = SyntaxTree::from_parser(parser, text);
			let approval_document = format!("{}\n===>\n{:#?}", text, tree);
			assert_snapshot_matches!(stringify!($name), approval_document);
		}
	)*
	}
}

#[test]
fn bar() {
	let text: &str = r#"
	let x = 1;"#;
	let tokens = tokenize(text);
	let mut parser = Parser::new(StrTokenSource::new(tokens.clone()));
	root(&mut parser);
	let tree = SyntaxTree::from_parser(parser.clone(), text);

	let approval_document = format!("====text====\n============{}\n\
				====tokens====\n============{:#?}\
				====events====\n============{:#?}\
				====tree====\n============\n{:#?}", text, tokens, parser.end_parsing(), tree);
	assert_snapshot_matches!(stringify!($name), approval_document);
}
// associativity & precedence
syntax_tree_expr_tests! {
	left_associativity_is_deeply_nested: "1+2+3",
	higher_precedence_is_evaluated_first: "1+2*3",
	higher_precedence_is_noop_when_first: "1*2+3",
	unary_operators_are_properly_grouped: "-1*2+-3",
	nested_unary_operators: "1*--2.12",
	grouping_is_highest_precedence: "(1+2)*3",
	starting_whitespace_is_fine: r#"
	1*2"#,
}

syntax_tree_stmt_tests! {
	starting_whitespace_is_fine: r#"
	let x = 1;
	let y = 2;
	let c = x * 2 + y;
	"#,
}

// operators 
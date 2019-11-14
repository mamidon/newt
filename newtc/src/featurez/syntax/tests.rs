#![cfg(test)]

use super::*;

use insta::assert_snapshot_matches;

use crate::featurez::grammar::{root_expr, root_stmt};
use crate::featurez::parse::Parser;
use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::TokenKind;
use crate::featurez::newtypes::TransparentNewType;

#[test]
fn return_stmt_node_includes_result_expr() {
	let tree: SyntaxTree = "return 42;".into();
	let return_node: &ReturnStmtNode = expect_node(&tree);

	return_node.result().expect("Valid result expr");
}

#[test]
fn return_stmt_node_does_not_require_result_expr() {
	let tree: SyntaxTree = "return;".into();
	let return_node: &ReturnStmtNode = expect_node(&tree);

	assert!(return_node.result().is_none());
}

#[test]
fn return_stmt_node_can_round_trip_as_stmt_node() {
	let tree: SyntaxTree = "return;".into();
	let return_node: &ReturnStmtNode = expect_node(&tree);

	let stmt_node = StmtNode::cast(return_node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::ReturnStmt(_) => {},
		_ => panic!("Could not round trip ReturnStmt as Stmt")
	};
}

#[test]
fn function_declaration_stmt_node_has_identifier() {
	let tree: SyntaxTree = "fn foo() {}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_node(&tree);

	assert_eq!("foo", function_node.identifier().lexeme());
}

#[test]
fn function_declaration_stmt_node_handles_zero_parameters() {
	let tree: SyntaxTree = "fn foo() {}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_node(&tree);

	assert_eq!(0, function_node.arguments().count());
}

#[test]
fn function_declaration_stmt_node_handles_1_parameter() {
	let tree: SyntaxTree = "fn foo(x) {}".into();
	let function_node: &FunctionDeclarationStmtNode = expect_node(&tree);

	assert_eq!(1, function_node.arguments().count());
	assert_eq!("x", function_node.arguments().nth(0).unwrap().lexeme());
}


#[test]
fn function_declaration_stmt_node_handles_multiple_parameter() {
	let tree: SyntaxTree = "fn foo(x, y, z) {}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_node(&tree);

	assert_eq!(3, function_node.arguments().count());
	assert_eq!("x", function_node.arguments().nth(0).unwrap().lexeme());
	assert_eq!("y", function_node.arguments().nth(1).unwrap().lexeme());
	assert_eq!("z", function_node.arguments().nth(2).unwrap().lexeme());
}

fn expect_node<N: TransparentNewType<Inner=SyntaxNode>>(tree: &SyntaxTree) -> &N {
	tree.root()
		.as_node()
		.map(|n| N::from_inner(n))
		.expect("Expected a root node with a valid type")
}



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
    {let x = 1;
	let y = 2;
	let c = x * 2 + y;}
	"#,
}



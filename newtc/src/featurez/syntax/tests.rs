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
	let return_node: &ReturnStmtNode = expect_stmt_node(&tree);

	return_node.result().expect("Valid result expr");
}

#[test]
fn return_stmt_node_does_not_require_result_expr() {
	let tree: SyntaxTree = "return;".into();
	let return_node: &ReturnStmtNode = expect_stmt_node(&tree);

	assert!(return_node.result().is_none());
}

#[test]
fn return_stmt_node_can_round_trip_as_stmt_node() {
	let tree: SyntaxTree = "return;".into();
	let return_node: &ReturnStmtNode = expect_stmt_node(&tree);

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

	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!("foo", function_node.identifier().lexeme());
}

#[test]
fn function_declaration_stmt_node_handles_zero_parameters() {
	let tree: SyntaxTree = "fn foo() {}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(0, function_node.arguments().count());
}

#[test]
fn function_declaration_stmt_node_handles_1_parameter() {
	let tree: SyntaxTree = "fn foo(x) {}".into();
	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(1, function_node.arguments().count());
	assert_eq!("x", function_node.arguments().nth(0).unwrap().lexeme());
}


#[test]
fn function_declaration_stmt_node_handles_multiple_parameter() {
	let tree: SyntaxTree = "fn foo(x, y, z) {}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(3, function_node.arguments().count());
	assert_eq!("x", function_node.arguments().nth(0).unwrap().lexeme());
	assert_eq!("y", function_node.arguments().nth(1).unwrap().lexeme());
	assert_eq!("z", function_node.arguments().nth(2).unwrap().lexeme());
}

#[test]
fn function_declaration_stmt_node_handles_zero_statements() {
	let tree: SyntaxTree = "fn foo(x, y, z) {}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(0, function_node.stmts().stmts().count());
}

#[test]
fn function_declaration_stmt_node_handles_single_statement() {
	let tree: SyntaxTree = "fn foo(x, y, z) { return 42; }".into();

	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(1, function_node.stmts().stmts().count());
}

#[test]
fn function_declaration_stmt_node_handles_multiple_statement() {
	let tree: SyntaxTree = "fn foo(x, y, z) { \
	    if (x > y) {\
			return x;
	    }\
	    return y;\
	}".into();

	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(2, function_node.stmts().stmts().count());
}

#[test]
fn function_declaration_stmt_node_round_trips() {
	let tree: SyntaxTree = "fn foo(x, y, z) {}".into();
	let function_node: &FunctionDeclarationStmtNode = expect_stmt_node(&tree);

	let stmt_node = StmtNode::cast(function_node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::FunctionDeclarationStmt(_) => {},
		_ => panic!("Could not round trip FunctionDeclarationStmtNode as Stmt")
	};
}

#[test]
fn while_stmt_node_handles_conditional() {
	let tree: SyntaxTree = "while(true) {}".into();
	let while_node: &WhileStmtNode = expect_stmt_node(&tree);

	let conditional_kind = while_node.condition()
		.syntax()
		.nth_node(0)
		.kind();

	assert_eq!(SyntaxKind::LiteralExpr, conditional_kind);
}

#[test]
fn while_stmt_node_handles_stmts() {
	let tree: SyntaxTree = "while(true) {}".into();
	let while_node: &WhileStmtNode = expect_stmt_node(&tree);

	let stmt_count = while_node.stmts()
		.stmts()
		.count();

	assert_eq!(0, stmt_count);
}

#[test]
fn while_stmt_node_round_trips() {
	let tree: SyntaxTree = "while (true) {}".into();
	let while_node: &WhileStmtNode = expect_stmt_node(&tree);

	let stmt_node = StmtNode::cast(while_node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::WhileStmt(_) => {},
		_ => panic!("Could not round trip Stmt")
	};
}

#[test]
fn if_stmt_node_handles_conditional() {
	let tree: SyntaxTree = "if(true) {}".into();
	let node: &IfStmtNode = expect_stmt_node(&tree);

	let conditional_kind = node.condition()
		.syntax()
		.nth_node(0)
		.kind();

	assert_eq!(SyntaxKind::LiteralExpr, conditional_kind);
}

#[test]
fn if_stmt_node_handles_when_true_stmts() {
	let tree: SyntaxTree = "if(true) {}".into();
	let node: &IfStmtNode = expect_stmt_node(&tree);

	let stmt_count= node.when_true()
		.stmts()
		.count();

	assert_eq!(0, stmt_count);
}

#[test]
fn if_stmt_node_handles_when_false_stmts() {
	let tree: SyntaxTree = "if(true) {} else {}".into();
	let node: &IfStmtNode = expect_stmt_node(&tree);

	let stmt_count= node.when_false()
		.unwrap()
		.stmts()
		.count();

	assert_eq!(0, stmt_count);
}

#[test]
fn if_stmt_node_handles_when_false_stmts_not_provided() {
	let tree: SyntaxTree = "if(true) {}".into();
	let node: &IfStmtNode = expect_stmt_node(&tree);

	assert!(node.when_false().is_none());
}

#[test]
fn if_stmt_node_does_not_swap_true_and_false_branches() {
	let tree: SyntaxTree = "if(true) \
	{ let x = 1; } \
	else { let y = 2; let z = 3; }".into();
	let node: &IfStmtNode = expect_stmt_node(&tree);

	let true_stmt_count= node.when_true()
		.stmts()
		.count();

	let false_stmt_count = node.when_false()
		.unwrap()
		.stmts()
		.count();

	assert_eq!(1, true_stmt_count);
	assert_eq!(2, false_stmt_count);
}

#[test]
fn if_stmt_node_round_trips() {
	let tree: SyntaxTree = "if (true) {}".into();
	let node: &IfStmtNode = expect_stmt_node(&tree);

	let stmt_node = StmtNode::cast(node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::IfStmt(_) => {},
		_ => panic!("Could not round trip Stmt")
	};
}

#[test]
fn variable_declaration_stmt_node_handles_identifier() {
	let tree: SyntaxTree = "let x = 42;".into();
	let node: &VariableDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!("x", node.identifier().lexeme());
}

#[test]
fn variable_declaration_stmt_node_handles_expr() {
	let tree: SyntaxTree = "let x = 42;".into();
	let node: &VariableDeclarationStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::LiteralExpr, node.expr().syntax().kind());
}

#[test]
fn variable_declaration_stmt_node_round_trips() {
	let tree: SyntaxTree = "let x = 42;".into();
	let node: &VariableDeclarationStmtNode = expect_stmt_node(&tree);

	let stmt_node = StmtNode::cast(node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::VariableDeclarationStmt(_) => {},
		_ => panic!("Could not round trip Stmt")
	};
}

#[test]
fn stmt_list_stmt_node_handles_zero_stmts() {
	let tree: SyntaxTree = "{}".into();
	let node: &StmtListStmtNode = expect_stmt_node(&tree);

	assert_eq!(0, node.stmts().count());
}

#[test]
fn stmt_list_stmt_node_handles_one_stmts() {
	let tree: SyntaxTree = "{ let x = 1; }".into();
	let node: &StmtListStmtNode = expect_stmt_node(&tree);

	assert_eq!(1, node.stmts().count());
}

#[test]
fn stmt_list_stmt_node_handles_multiple_stmts() {
	let tree: SyntaxTree = "{ let x = 1; let y = 2; let x = 3; }".into();
	let node: &StmtListStmtNode = expect_stmt_node(&tree);

	assert_eq!(3, node.stmts().count());
}

#[test]
fn stmt_list_stmt_node_round_trips() {
	let tree: SyntaxTree = "{}".into();
	let node: &StmtListStmtNode = expect_stmt_node(&tree);

	let stmt_node = StmtNode::cast(node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::StmtListStmt(_) => {},
		_ => panic!("Could not round trip Stmt")
	};
}

#[test]
fn expr_stmt_node_handles_literal_expr() {
	let tree: SyntaxTree = "42;".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::LiteralExpr, node.expr().syntax().kind());
}

#[test]
fn expr_stmt_node_handles_binary_expr() {
	let tree: SyntaxTree = "2 + 2;".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::BinaryExpr, node.expr().syntax().kind());
}

#[test]
fn expr_stmt_node_handles_unary_expr() {
	let tree: SyntaxTree = "!false;".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::UnaryExpr, node.expr().syntax().kind());
}

#[test]
fn expr_stmt_node_handles_grouping_expr() {
	let tree: SyntaxTree = "(42);".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::GroupingExpr, node.expr().syntax().kind());
}

#[test]
fn expr_stmt_node_handles_variable_expr() {
	let tree: SyntaxTree = "x;".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::VariableExpr, node.expr().syntax().kind());
}

#[test]
fn expr_stmt_node_handles_function_call_expr() {
	let tree: SyntaxTree = "foo();".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	assert_eq!(SyntaxKind::FunctionCallExpr, node.expr().syntax().kind());
}

#[test]
fn expr_stmt_node_round_trips() {
	let tree: SyntaxTree = "42;".into();
	let node: &ExprStmtNode = expect_stmt_node(&tree);

	let stmt_node = StmtNode::cast(node.to_inner())
		.expect("Valid StmtNode");

	match stmt_node.kind() {
		StmtKind::ExprStmt(_) => {},
		_ => panic!("Could not round trip Stmt")
	};
}

#[test]
fn function_call_expr_node_handles_callee() {
	let tree: SyntaxTree = "foo();".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::VariableExpr, node.callee().syntax().kind());
}

#[test]
fn function_call_expr_node_handles_zero_arguments() {
	let tree: SyntaxTree = "foo();".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(0, node.arguments().count());
}

#[test]
fn function_call_expr_node_handles_one_arguments() {
	let tree: SyntaxTree = "foo(x);".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(1, node.arguments().count());
}

#[test]
fn function_call_expr_node_handles_multiple_arguments() {
	let tree: SyntaxTree = "foo(x, y, z);".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(3, node.arguments().count());
}


#[test]
fn function_call_expr_node_properly_orders_arguments() {
	let tree: SyntaxTree = "foo(x, 2+2, bar());".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);
	let arguments: Vec<&ExprNode> = node.arguments().collect();

	assert_eq!(3, arguments.len());
	assert_eq!(SyntaxKind::VariableExpr, arguments[0].syntax().kind());
	assert_eq!(SyntaxKind::BinaryExpr, arguments[1].syntax().kind());
	assert_eq!(SyntaxKind::FunctionCallExpr, arguments[2].syntax().kind());
}


fn expect_stmt_node<N: TransparentNewType<Inner=SyntaxNode>>(tree: &SyntaxTree) -> &N {
	tree.root()
		.as_node()
		.map(|n| N::from_inner(n))
		.expect("Expected a root node with a valid type")
}

fn expect_expr_node<N: TransparentNewType<Inner=SyntaxNode>>(tree: &SyntaxTree) -> &N {
	let stmt: &ExprStmtNode = expect_stmt_node(tree);
	N::from_inner(stmt.expr().syntax())
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



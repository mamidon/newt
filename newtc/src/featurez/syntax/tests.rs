#![cfg(test)]

use super::*;

use insta::assert_snapshot_matches;

use crate::featurez::grammar::{root_expr, root_stmt};
use crate::featurez::parse::Parser;
use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::TokenKind;
use crate::featurez::newtypes::TransparentNewType;
use std::collections::HashMap;

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

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, conditional_kind);
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

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, conditional_kind);
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

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.expr().syntax().kind());
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

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.expr().syntax().kind());
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
	let tree: SyntaxTree = "foo()".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::VariableExpr, node.callee().syntax().kind());
}

#[test]
fn function_call_expr_node_handles_zero_arguments() {
	let tree: SyntaxTree = "foo()".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(0, node.arguments().count());
}

#[test]
fn function_call_expr_node_handles_one_arguments() {
	let tree: SyntaxTree = "foo(x)".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(1, node.arguments().count());
}

#[test]
fn function_call_expr_node_handles_multiple_arguments() {
	let tree: SyntaxTree = "foo(x, y, z)".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	assert_eq!(3, node.arguments().count());
}

#[test]
fn function_call_expr_node_properly_orders_arguments() {
	let tree: SyntaxTree = "foo(x, 2+2, bar())".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);
	let arguments: Vec<&ExprNode> = node.arguments().collect();

	assert_eq!(3, arguments.len());
	assert_eq!(SyntaxKind::VariableExpr, arguments[0].syntax().kind());
	assert_eq!(SyntaxKind::BinaryExpr, arguments[1].syntax().kind());
	assert_eq!(SyntaxKind::FunctionCallExpr, arguments[2].syntax().kind());
}

#[test]
fn function_call_expr_node_round_trips() {
	let tree: SyntaxTree = "foo()".into();
	let node: &FunctionCallExprNode = expect_expr_node(&tree);

	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::FunctionCallExpr(_) => {},
		_ => panic!("Could not round trip Stmt")
	};
}

#[test]
fn primitive_literal_expr_node_handles_integers() {
	let tree: SyntaxTree = "42".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.to_inner().kind());
}

#[test]
fn primitive_literal_expr_node_handles_true_booleans() {
	let tree: SyntaxTree = "true".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.to_inner().kind());
}

#[test]
fn primitive_literal_expr_node_handles_false_booleans() {
	let tree: SyntaxTree = "false".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.to_inner().kind());
}

#[test]
fn primitive_literal_expr_node_handles_string_booleans() {
	let tree: SyntaxTree = "\"hello, world!\"".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.to_inner().kind());
}

#[test]
fn primitive_literal_expr_node_handles_glyphs() {
	let tree: SyntaxTree = "'a'".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.to_inner().kind());
}

#[test]
fn primitive_literal_expr_node_handles_floats() {
	let tree: SyntaxTree = "3.14".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.to_inner().kind());
}

#[test]
fn primitive_literal_expr_node_round_trips() {
	let tree: SyntaxTree = "42".into();
	let node: &PrimitiveLiteralExprNode = expect_expr_node(&tree);

	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::PrimitiveLiteralExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}

#[test]
fn object_literal_expr_node_handles_zero_fields() {
	let tree: SyntaxTree = "let foo = {};".into();
	let stmt: &VariableDeclarationStmtNode = expect_stmt_node(&tree);
	let node: &ObjectLiteralExprNode = match stmt.expr().kind() {
		ExprKind::ObjectLiteralExpr(literal) => literal,
		_ => panic!("Could not parse object literal")
	};

	assert_eq!(0, node.fields().len());
}


#[test]
fn object_literal_expr_node_handles_one_field() {
	let tree: SyntaxTree = "let foo = { bar: 42 };".into();
	let stmt: &VariableDeclarationStmtNode = expect_stmt_node(&tree);
	let node: &ObjectLiteralExprNode = match stmt.expr().kind() {
		ExprKind::ObjectLiteralExpr(literal) => literal,
		_ => panic!("Could not parse object literal")
	};

	assert_eq!(1, node.fields().len());
	assert_eq!(Some(SyntaxKind::PrimitiveLiteralExpr), node.fields().get("bar").map(|e| e.to_inner().kind()));
}

#[test]
fn object_literal_expr_node_handles_multiple_fields() {
	let tree: SyntaxTree = "let foo = { bar: 42, fizz: 2+2, buzz: other_variable };".into();
	let stmt: &VariableDeclarationStmtNode = expect_stmt_node(&tree);
	let node: &ObjectLiteralExprNode = match stmt.expr().kind() {
		ExprKind::ObjectLiteralExpr(literal) => literal,
		_ => panic!("Could not parse object literal")
	};
	let fields: HashMap<String, SyntaxKind> = node.fields()
		.into_iter()
		.map(|(k, v)| (k, v.to_inner().kind()))
		.collect();

	assert_eq!(3, fields.len());
	assert_eq!(Some(&SyntaxKind::PrimitiveLiteralExpr), fields.get("bar"));
	assert_eq!(Some(&SyntaxKind::BinaryExpr), fields.get("fizz"));
	assert_eq!(Some(&SyntaxKind::VariableExpr), fields.get("buzz"));
}

#[test]
fn object_literal_expr_node_handles_nested_fields() {
	let tree: SyntaxTree = "let foo = { bar: { fizz: 2+2, buzz: other_variable } };".into();
	let stmt: &VariableDeclarationStmtNode = expect_stmt_node(&tree);
	let outer_node: &ObjectLiteralExprNode = match stmt.expr().kind() {
		ExprKind::ObjectLiteralExpr(literal) => literal,
		_ => panic!("Could not parse object literal")
	};
	let inner_fields = match outer_node.fields().get("bar") {
		Some(expr) => match expr.kind() {
			ExprKind::ObjectLiteralExpr(literal) => {
				let bar_fields: HashMap<String, SyntaxKind> = literal.fields()
					.into_iter()
					.map(|(k, v)| (k, v.to_inner().kind()))
					.collect();
				bar_fields
			},
			_ => panic!("Could not parse object literal")
		},
		_ => panic!("Could not parse nested object literals")
	};

	assert_eq!(2, inner_fields.len());
	assert_eq!(Some(&SyntaxKind::BinaryExpr), inner_fields.get("fizz"));
	assert_eq!(Some(&SyntaxKind::VariableExpr), inner_fields.get("buzz"));
}

#[test]
fn object_literal_expr_node_round_trips() {
	let tree: SyntaxTree = "let x = {};".into();
	let stmt: &VariableDeclarationStmtNode = expect_stmt_node(&tree);
	let node: &ObjectLiteralExprNode = match stmt.expr().kind() {
		ExprKind::ObjectLiteralExpr(literal) => literal,
		_ => panic!("Could not parse object literal")
	};
	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::ObjectLiteralExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}

#[test]
fn object_property_expr_handles_single_property_access() {
	let tree: SyntaxTree = "fizz.buzz".into();
	let expr: &ObjectPropertyExprNode = expect_expr_node(&tree);

	assert_eq!("buzz", expr.identifier().lexeme());
}

#[test]
fn object_property_expr_handles_multiple_property_access() {
	let tree: SyntaxTree = "fizz.buzz.foo".into();
	let foo_expr: &ObjectPropertyExprNode = expect_expr_node(&tree);
	let buzz_expr: &ObjectPropertyExprNode = ObjectPropertyExprNode::from_inner(foo_expr.source_expr().to_inner());
	let fizz_expr: &VariableExprNode = VariableExprNode::from_inner(buzz_expr.source_expr().to_inner());

	assert_eq!("foo", foo_expr.identifier().lexeme());
	assert_eq!("buzz", buzz_expr.identifier().lexeme());
	assert_eq!("fizz", fizz_expr.identifier().lexeme());
}

#[test]
fn object_property_expr_node_round_trips() {
	let tree: SyntaxTree = "fizz.buzz".into();
	let expr: &ExprNode = ExprNode::from_inner(tree.root().as_node().unwrap());
	let node: &ObjectPropertyExprNode = match expr.kind() {
		ExprKind::ObjectPropertyExpr(literal) => literal,
		_ => panic!("Could not parse object property expression")
	};
	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::ObjectPropertyExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}

#[test]
fn binary_expr_node_does_not_swap_operands() {
	let tree: SyntaxTree = "2+foo()".into();
	let node: &BinaryExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.lhs().syntax().kind());
	assert_eq!(SyntaxKind::FunctionCallExpr, node.rhs().syntax().kind());
}

#[test]
fn binary_expr_node_handles_operators() {
	fn binary_expr_node_operator_test(source: &str, expected_operator: TokenKind) {
		let tree: SyntaxTree = source.into();
		let node: &BinaryExprNode = expect_expr_node(&tree);

		assert_eq!(expected_operator, node.operator(), "{}", source);
	}

	let test_cases = [
		("2+2", TokenKind::Plus),
		("2-2", TokenKind::Minus),
		("2*2", TokenKind::Star),
		("2/2", TokenKind::Slash),

		("2>2", TokenKind::Greater),
		("2>=2", TokenKind::GreaterEquals),
		("2<2", TokenKind::Less),
		("2<=2", TokenKind::LessEquals),

		("2==2", TokenKind::EqualsEquals),
	];

	for test_case in &test_cases {
		binary_expr_node_operator_test(test_case.0, test_case.1);
	}
}

#[test]
fn binary_expr_node_round_trips() {
	let tree: SyntaxTree = "2+2".into();
	let node: &BinaryExprNode = expect_expr_node(&tree);

	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::BinaryExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}

#[test]
fn unary_expr_node_handles_operand() {
	let tree: SyntaxTree = "-2".into();
	let node: &UnaryExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::PrimitiveLiteralExpr, node.rhs().syntax().kind());
}

#[test]
fn unary_expr_node_handles_operators() {
	fn unary_expr_node_operator_test(source: &str, expected_operator: TokenKind) {
		let tree: SyntaxTree = source.into();
		let node: &UnaryExprNode = expect_expr_node(&tree);

		assert_eq!(expected_operator, node.operator(), "{}", source);
	}

	let test_cases = [
		("!true", TokenKind::Bang),
		("-2", TokenKind::Minus),
	];

	for test_case in &test_cases {
		unary_expr_node_operator_test(test_case.0, test_case.1);
	}
}

#[test]
fn unary_expr_node_round_trips() {
	let tree: SyntaxTree = "-2".into();
	let node: &BinaryExprNode = expect_expr_node(&tree);

	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::UnaryExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}

#[test]
fn grouping_expr_handles_sub_expr() {
	let tree: SyntaxTree = "(2+2)".into();
	let node: &GroupingExprNode = expect_expr_node(&tree);

	assert_eq!(SyntaxKind::BinaryExpr, node.expr().syntax().kind());
}

#[test]
fn grouping_expr_node_round_trips() {
	let tree: SyntaxTree = "(2+2)".into();
	let node: &GroupingExprNode = expect_expr_node(&tree);

	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::GroupingExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}

#[test]
fn variable_expr_node_handles_identifier() {
	let tree: SyntaxTree = "x".into();
	let node: &VariableExprNode = expect_expr_node(&tree);

	assert_eq!("x", node.identifier().lexeme());
}

#[test]
fn variable_expr_node_round_trips() {
	let tree: SyntaxTree = "x".into();
	let node: &VariableExprNode = expect_expr_node(&tree);

	let expr = ExprNode::cast(node.to_inner()).unwrap();

	match expr.kind() {
		ExprKind::VariableExpr(_) => {},
		_ => panic!("Could not round trip Expr")
	};
}


fn expect_stmt_node<N: TransparentNewType<Inner=SyntaxNode>>(tree: &SyntaxTree) -> &N {
	tree.root()
		.as_node()
		.map(|r| StmtListStmtNode::from_inner(r))
		.map(|n| N::from_inner(n.stmts().nth(0).unwrap().to_inner()))
		.expect("Expected a root node with a valid type")
}

fn expect_expr_node<N: TransparentNewType<Inner=SyntaxNode>>(tree: &SyntaxTree) -> &N {
	tree.root()
		.as_node()
		.map(|r| ExprNode::from_inner(r))
		.map(|n| N::from_inner(n.to_inner()))
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



use crate::featurez::TokenKind;
use crate::featurez::syntax::{
	BinaryExprNode, UnaryExprNode, LiteralExprNode, AstNode, ExprNode, ExprKind,
	SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
	SyntaxTree, TextTreeSink, TokenSource, TreeSink,
};
use std::ops::{Add, Sub, Mul, Div, Not, Neg};
use std::str::FromStr;

pub trait ExprVisitor<T>
{
	fn visit_expr(&self, expr: &ExprNode) -> T {
		match expr.kind() {
			ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
			ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
			ExprKind::LiteralExpr(node) => self.visit_literal_expr(node)
		}
	}

	fn visit_binary_expr(&self, node: &BinaryExprNode) -> T;
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> T;
	fn visit_literal_expr(&self, node: &LiteralExprNode) -> T;
}

pub struct ExprVirtualMachine {}

impl ExprVirtualMachine {
	pub fn new() -> ExprVirtualMachine { ExprVirtualMachine {} }
}

impl<T> ExprVisitor<T> for ExprVirtualMachine
	where T: Add<Output=T>
	+ Sub<Output=T>
	+ Mul<Output=T>
	+ Div<Output=T>
	+ Not<Output=T>
	+ Neg<Output=T>
	+ FromStr {
	fn visit_binary_expr(&self, node: &BinaryExprNode) -> T {
		let lhs: T = self.visit_expr(node.lhs());
		let rhs: T = self.visit_expr(node.rhs());

		match node.operator() {
			TokenKind::Plus => lhs + rhs,
			TokenKind::Minus => lhs - rhs,
			TokenKind::Star => lhs * rhs,
			TokenKind::Slash => lhs / rhs,
			_ => unreachable!("not a binary")
		}
	}

	fn visit_unary_expr(&self, node: &UnaryExprNode) -> T {
		let rhs: T = self.visit_expr(node.rhs());

		match node.operator() {
			TokenKind::Bang => !rhs,
			TokenKind::Minus => -rhs,
			_ => unreachable!("not a unary")
		}
	}

	fn visit_literal_expr(&self, node: &LiteralExprNode) -> T {
		let literal = node.literal();
		let value = literal.lexeme().parse::<T>();

		match value {
			Ok(v) => v,
			Err(_)=> unimplemented!()
		}
	}
}
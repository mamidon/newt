use crate::featurez::TokenKind;
use crate::featurez::syntax::{
	ExprKind, 
	ExprNode, 
	GroupingExprNode, 
	BinaryExprNode, 
	UnaryExprNode,
	LiteralExprNode
};

use super::NewtResult;
use super::NewtValue;

pub trait ExprVisitor
{
	fn visit_expr(&self, expr: &ExprNode) -> NewtResult {
		match expr.kind() {
			ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
			ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
			ExprKind::LiteralExpr(node) => self.visit_literal_expr(node),
			ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
		}
	}

	fn visit_binary_expr(&self, node: &BinaryExprNode) -> NewtResult;
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> NewtResult;
	fn visit_literal_expr(&self, node: &LiteralExprNode) -> NewtResult;
	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> NewtResult;
}

pub struct ExprVirtualMachine {}

impl ExprVirtualMachine {
	pub fn new() -> ExprVirtualMachine { ExprVirtualMachine {} }
}

impl ExprVisitor for ExprVirtualMachine {
	fn visit_binary_expr(&self, node: &BinaryExprNode) -> NewtResult {
		let lhs = self.visit_expr(node.lhs())?;
		let rhs = self.visit_expr(node.rhs())?;

		match node.operator() {
			TokenKind::Plus => lhs + rhs,
			TokenKind::Minus => lhs - rhs,
			TokenKind::Star => lhs * rhs,
			TokenKind::Slash => lhs / rhs,
			_ => unreachable!("not a binary")
		}
	}


	//noinspection RsTypeCheck -- faulty on the match statement
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> NewtResult {
		let rhs = self.visit_expr(node.rhs())?;
		
		match node.operator() {
			TokenKind::Bang => !rhs,
			TokenKind::Minus => -rhs,
			_ => unreachable!("not a unary")
		}
	}

	fn visit_literal_expr(&self, node: &LiteralExprNode) -> NewtResult {
		let literal = node.literal();
		let value = NewtValue::from_literal_node(node);

		Ok(value)
	}

	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> NewtResult {
		let expr = node.expr();

		self.visit_expr(expr)
	}
}

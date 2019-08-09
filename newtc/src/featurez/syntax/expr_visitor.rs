use super::*;

pub trait ExprVisitor
{
	fn visit_expr(&self, expr: &ExprNode) -> NewtResult {
		match expr.kind() {
			ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
			ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
			ExprKind::LiteralExpr(node) => self.visit_literal_expr(node),
			ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
			ExprKind::VariableExpr(node) => self.visit_variable_expr(node),
		}
	}

	fn visit_binary_expr(&self, node: &BinaryExprNode) -> NewtResult;
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> NewtResult;
	fn visit_literal_expr(&self, node: &LiteralExprNode) -> NewtResult;
	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> NewtResult;
	fn visit_variable_expr(&self, node: &VariableExprNode) -> NewtResult;
}

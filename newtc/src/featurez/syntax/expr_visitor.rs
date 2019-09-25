use super::*;

pub trait ExprVisitor
{
	fn visit_expr(&self, expr: &ExprNode) -> NewtResult;
	fn visit_binary_expr(&self, node: &BinaryExprNode) -> NewtResult;
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> NewtResult;
	fn visit_literal_expr(&self, node: &LiteralExprNode) -> NewtResult;
	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> NewtResult;
	fn visit_variable_expr(&self, node: &VariableExprNode) -> NewtResult;
	fn visit_function_call_expr(&self, node: &FunctionCallExprNode) -> NewtResult;
}

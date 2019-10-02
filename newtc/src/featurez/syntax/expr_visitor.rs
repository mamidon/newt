use super::*;

pub trait ExprVisitor<'a, R> {
    fn visit_expr(&mut self, expr: &'a ExprNode) -> R;
    fn visit_binary_expr(&mut self, node: &'a BinaryExprNode) -> R;
    fn visit_unary_expr(&mut self, node: &'a UnaryExprNode) -> R;
    fn visit_literal_expr(&mut self, node: &'a LiteralExprNode) -> R;
    fn visit_grouping_expr(&mut self, node: &'a GroupingExprNode) -> R;
    fn visit_variable_expr(&mut self, node: &'a VariableExprNode) -> R;
    fn visit_function_call_expr(&mut self, node: &'a FunctionCallExprNode) -> R;
}

use super::*;

pub trait ExprVisitor<R> {
    fn visit_expr(&mut self, expr: &ExprNode) -> R;
    fn visit_binary_expr(&mut self, node: &BinaryExprNode) -> R;
    fn visit_unary_expr(&mut self, node: &UnaryExprNode) -> R;
    fn visit_primitive_literal_expr(&mut self, node: &PrimitiveLiteralExprNode) -> R;
    fn visit_grouping_expr(&mut self, node: &GroupingExprNode) -> R;
    fn visit_variable_expr(&mut self, node: &VariableExprNode) -> R;
    fn visit_function_call_expr(&mut self, node: &FunctionCallExprNode) -> R;
    fn visit_object_literal_expr(&mut self, node: &ObjectLiteralExprNode) -> R;
    fn visit_object_property_expr(&mut self, node: &ObjectPropertyExprNode) -> R;
}

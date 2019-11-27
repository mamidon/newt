use crate::featurez::syntax::nodes::*;

pub enum ExprKind<'a> {
    BinaryExpr(&'a BinaryExprNode),
    UnaryExpr(&'a UnaryExprNode),
    PrimitiveLiteralExpr(&'a PrimitiveLiteralExprNode),
    GroupingExpr(&'a GroupingExprNode),
    VariableExpr(&'a VariableExprNode),
    FunctionCallExpr(&'a FunctionCallExprNode),
}

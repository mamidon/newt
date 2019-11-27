use crate::featurez::syntax::nodes::*;

pub enum ExprKind<'a> {
    BinaryExpr(&'a BinaryExprNode),
    UnaryExpr(&'a UnaryExprNode),
    PrimitiveLiteralExpr(&'a PrimitiveLiteralExprNode),
    ObjectLiteralExpr(&'a ObjectLiteralExprNode),
    GroupingExpr(&'a GroupingExprNode),
    VariableExpr(&'a VariableExprNode),
    FunctionCallExpr(&'a FunctionCallExprNode),
}

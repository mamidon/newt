use crate::featurez::syntax::nodes::*;

pub enum ExprKind<'a> {
	BinaryExpr(&'a BinaryExprNode),
	UnaryExpr(&'a UnaryExprNode),
	LiteralExpr(&'a LiteralExprNode)
}
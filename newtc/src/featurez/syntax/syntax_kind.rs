
#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
	TombStone,
	Expr,
	BinaryExpr,
	UnaryExpr,
	LiteralExpr
}

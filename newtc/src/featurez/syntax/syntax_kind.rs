
#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
	TombStone,
	PlusExpr,
	BinaryExpr,
	LiteralExpr
}

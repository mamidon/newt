#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
    TombStone,
	Error(&'static str),
    Expr,
    BinaryExpr,
    UnaryExpr,
    LiteralExpr,
}

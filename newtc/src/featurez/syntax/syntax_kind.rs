#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
    TombStone,
	Error(&'static str),
    BinaryExpr,
    UnaryExpr,
    LiteralExpr,
}


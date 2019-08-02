#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
    TombStone,
	Error(&'static str),
	GroupingExpr,
    BinaryExpr,
    UnaryExpr,
    LiteralExpr,
	VariableDeclarationStmt,
	VariableAssignmentStmt,
	StmtListStmt,
}


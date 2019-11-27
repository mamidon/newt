#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
    TombStone,
    Error(&'static str),
    GroupingExpr,
    BinaryExpr,
    UnaryExpr,
    PrimitiveLiteralExpr,
    VariableExpr,
    FunctionCallExpr,
    VariableDeclarationStmt,
    VariableAssignmentStmt,
    StmtListStmt,
    ExprStmt,
    IfStmt,
    WhileStmt,
    FunctionDeclarationStmt,
    ReturnStmt,
}

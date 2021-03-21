use std::fmt::{Display, Error, Formatter};

#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
    TombStone,
    Error(&'static str),
    GroupingExpr,
    BinaryExpr,
    UnaryExpr,
    PrimitiveLiteralExpr,
    ObjectLiteralExpr,
    ObjectPropertyExpr,
    ObjectPropertyRVal,
    VariableExpr,
    FunctionCallExpr,
    VariableDeclarationStmt,
    AssignmentStmt,
    VariableRval,
    StmtListStmt,
    ExprStmt,
    IfStmt,
    WhileStmt,
    FunctionDeclarationStmt,
    ReturnStmt,
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            SyntaxKind::Error(_) => write!(f, "Error"),
            syntax_kind => write!(f, "{:?}", syntax_kind),
        }
    }
}

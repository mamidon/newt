use crate::featurez::syntax::nodes::*;

pub enum StmtKind<'a> {
    VariableDeclarationStmt(&'a VariableDeclarationStmtNode),
    AssignmentStmt(&'a AssignmentStmtNode),
    StmtListStmt(&'a StmtListStmtNode),
    ExprStmt(&'a ExprStmtNode),
    IfStmt(&'a IfStmtNode),
    WhileStmt(&'a WhileStmtNode),
    FunctionDeclarationStmt(&'a FunctionDeclarationStmtNode),
    ReturnStmt(&'a ReturnStmtNode),
}

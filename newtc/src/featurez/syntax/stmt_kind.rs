use crate::featurez::syntax::nodes::*;

pub enum StmtKind<'a> {
	VariableDeclarationStmt(&'a VariableDeclarationStmtNode),
	VariableAssignmentStmt(&'a VariableAssignmentStmtNode),
	StmtListStmt(&'a StmtListStmtNode),
	ExprStmt(&'a ExprStmtNode),
	IfStmt(&'a IfStmtNode)
}
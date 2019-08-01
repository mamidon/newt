use super::*;

pub trait StmtVisitor
{
	fn visit_stmt(&self, stmt: &StmtNode) {
		match stmt.kind() {
			StmtKind::VariableDeclarationStmt(node) => self.visit_variable_declaration_stmt(node),
			StmtKind::VariableAssignmentStmt(node) => self.visit_variable_assignment_stmt(node)
		}
	}

	fn visit_variable_declaration_stmt(&self, node: &VariableDeclarationStmtNode);
	fn visit_variable_assignment_stmt(&self, node: &VariableAssignmentStmtNode);
}

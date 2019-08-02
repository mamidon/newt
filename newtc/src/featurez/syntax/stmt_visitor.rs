use super::*;

pub trait StmtVisitor
{
	fn visit_stmt(&mut self, stmt: &StmtNode) {
		match stmt.kind() {
			StmtKind::VariableDeclarationStmt(node) => self.visit_variable_declaration_stmt(node),
			StmtKind::VariableAssignmentStmt(node) => self.visit_variable_assignment_stmt(node)
		}
	}

	fn visit_variable_declaration_stmt(&mut self, node: &VariableDeclarationStmtNode);
	fn visit_variable_assignment_stmt(&mut self, node: &VariableAssignmentStmtNode);
}

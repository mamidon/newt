use super::*;

pub trait StmtVisitor
{
	fn visit_stmt(&mut self, stmt: &StmtNode) {
		match stmt.kind() {
			StmtKind::VariableDeclarationStmt(node) => self.visit_variable_declaration_stmt(node),
			StmtKind::VariableAssignmentStmt(node) => self.visit_variable_assignment_stmt(node),
			StmtKind::StmtListStmt(node) => self.visit_stmt_list_stmt(node),
			StmtKind::ExprStmt(node) => self.visit_expr_stmt(node),
			StmtKind::IfStmt(node) => self.visit_if_stmt(node),
			StmtKind::WhileStmt(node) => self.visit_while_stmt(node),
		}
	}

	fn visit_variable_declaration_stmt(&mut self, node: &VariableDeclarationStmtNode);
	fn visit_variable_assignment_stmt(&mut self, node: &VariableAssignmentStmtNode);
	fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode);
	fn visit_expr_stmt(&mut self, node: &ExprStmtNode);
	fn visit_if_stmt(&mut self, node: &IfStmtNode);
	fn visit_while_stmt(&mut self, node: &WhileStmtNode);
}

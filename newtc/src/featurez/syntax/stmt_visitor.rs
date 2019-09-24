use super::*;

pub trait StmtVisitor
{
	fn visit_stmt(&mut self, stmt: &StmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_variable_declaration_stmt(&mut self, node: &VariableDeclarationStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_variable_assignment_stmt(&mut self, node: &VariableAssignmentStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_expr_stmt(&mut self, node: &ExprStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_if_stmt(&mut self, node: &IfStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_while_stmt(&mut self, node: &WhileStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_function_declaration_stmt(&mut self, node: &FunctionDeclarationStmtNode) -> Result<(), NewtRuntimeError>;
	fn visit_return_stmt(&mut self, node: &ReturnStmtNode) -> Result<(), NewtRuntimeError>;
}

use super::*;

pub trait StmtVisitor<'a, T> {
    fn visit_stmt(&mut self, stmt: &'a StmtNode) -> T;
    fn visit_variable_declaration_stmt(&mut self, node: &'a VariableDeclarationStmtNode) -> T;
    fn visit_variable_assignment_stmt(&mut self, node: &'a VariableAssignmentStmtNode) -> T;
    fn visit_stmt_list_stmt(&mut self, node: &'a StmtListStmtNode) -> T;
    fn visit_expr_stmt(&mut self, node: &'a ExprStmtNode) -> T;
    fn visit_if_stmt(&mut self, node: &'a IfStmtNode) -> T;
    fn visit_while_stmt(&mut self, node: &'a WhileStmtNode) -> T;
    fn visit_function_declaration_stmt(&mut self, node: &'a FunctionDeclarationStmtNode) -> T;
    fn visit_return_stmt(&mut self, node: &'a ReturnStmtNode) -> T;
}

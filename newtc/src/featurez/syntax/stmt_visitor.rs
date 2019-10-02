use super::*;

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: &StmtNode) -> T;
    fn visit_variable_declaration_stmt(
        &mut self,
        node: &VariableDeclarationStmtNode,
    ) -> T;
    fn visit_variable_assignment_stmt(
        &mut self,
        node: &VariableAssignmentStmtNode,
    ) -> T;
    fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode) -> T;
    fn visit_expr_stmt(&mut self, node: &ExprStmtNode) -> T;
    fn visit_if_stmt(&mut self, node: &IfStmtNode) -> T;
    fn visit_while_stmt(&mut self, node: &WhileStmtNode) -> T;
    fn visit_function_declaration_stmt(
        &mut self,
        node: &FunctionDeclarationStmtNode,
    ) -> T;
    fn visit_return_stmt(&mut self, node: &ReturnStmtNode) -> T;
}

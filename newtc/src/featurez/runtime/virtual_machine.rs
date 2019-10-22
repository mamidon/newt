use super::scope::LexicalScope;
use crate::featurez::syntax::*;
use crate::featurez::TokenKind;
use std::collections::HashMap;
use crate::featurez::runtime::{RefEquality, Callable};
use crate::featurez::newtypes::TransparentNewType;

#[derive(Debug)]
pub struct VirtualMachineState {
    scope: LexicalScope,
    stack: Vec<Box<Callable>>,
    halting_error: Option<NewtRuntimeError>,
    tree: Option<SyntaxTree>
}

impl VirtualMachineState {
    pub fn new() -> VirtualMachineState {
        VirtualMachineState {
            scope: LexicalScope::new(),
            stack: Vec::new(),
            halting_error: None,
            tree: None
        }
    }

    pub fn visit_stmt_list_stmt_with_scope(&mut self, node: &StmtListStmtNode, scope: &mut LexicalScope) -> Result<(), NewtRuntimeError> {
        std::mem::swap(&mut self.scope, scope);

        self.visit_stmt_list_stmt(node);

        std::mem::swap(&mut self.scope, scope);

        Ok(())
    }

    fn halt(&mut self, error: NewtRuntimeError) -> Result<(), NewtRuntimeError> {
        if !self.halted() {
            self.halting_error = Some(error)
        }

        Err(self.halting_error.unwrap())
    }

    fn halt_on_error<T>(
        &mut self,
        result: Result<T, NewtRuntimeError>,
    ) -> Result<T, NewtRuntimeError> {
        if let Some(error) = self.halting_error {
            return Err(error);
        }

        if let Err(error) = result {
            self.halt(error);
        }

        result
    }

    fn halted(&self) -> bool {
        self.halting_error.is_some()
    }
}

pub struct VirtualMachineInterpretingSession<'sess> {
    tree: &'sess SyntaxTree,
    state: &'sess mut VirtualMachineState,
}

impl<'sess> VirtualMachineInterpretingSession<'sess> {
    pub fn new(tree: &'sess SyntaxTree,
               state: &'sess mut VirtualMachineState)
        -> VirtualMachineInterpretingSession<'sess> {

        VirtualMachineInterpretingSession {
            tree,
            state,
        }
    }

    pub fn interpret(&mut self) -> Option<NewtValue> {
        let node = match self.tree.root().as_node() {
            Some(n) => n,
            None => return None,
        };

        if let Some(expr) = ExprNode::cast(node) {
            return self.state.visit_expr(expr).ok();
        }

        if let Some(stmt) = StmtNode::cast(node) {
            self.state.visit_stmt(stmt);

            return None;
        }

        return None;
    }
}

impl ExprVisitor<NewtResult> for VirtualMachineState {
    fn visit_expr(&mut self, node: &ExprNode) -> NewtResult {
        if let Some(error) = self.halting_error {
            return Err(error);
        }

        let outcome = match node.kind() {
            ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
            ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
            ExprKind::LiteralExpr(node) => self.visit_literal_expr(node),
            ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
            ExprKind::VariableExpr(node) => self.visit_variable_expr(node),
            ExprKind::FunctionCallExpr(node) => self.visit_function_call_expr(node),
        };

        outcome
    }

    fn visit_binary_expr(&mut self, node: &BinaryExprNode) -> NewtResult {
        let lhs = self.visit_expr(node.lhs())?;
        let rhs = self.visit_expr(node.rhs())?;

        match node.operator() {
            TokenKind::Plus => lhs + rhs,
            TokenKind::Minus => lhs - rhs,
            TokenKind::Star => lhs * rhs,
            TokenKind::Slash => lhs / rhs,
            TokenKind::Greater => Ok(NewtValue::Bool(lhs > rhs)),
            TokenKind::GreaterEquals => Ok(NewtValue::Bool(lhs >= rhs)),
            TokenKind::Less => Ok(NewtValue::Bool(lhs < rhs)),
            TokenKind::LessEquals => Ok(NewtValue::Bool(lhs <= rhs)),
            TokenKind::EqualsEquals => Ok(NewtValue::Bool(lhs == rhs)),
            _ => unreachable!("not a binary"),
        }
    }

    //noinspection RsTypeCheck -- faulty on the match statement
    fn visit_unary_expr(&mut self, node: &UnaryExprNode) -> NewtResult {
        let rhs = self.visit_expr(node.rhs())?;

        match node.operator() {
            TokenKind::Bang => !rhs,
            TokenKind::Minus => -rhs,
            _ => unreachable!("not a unary"),
        }
    }

    fn visit_literal_expr(&mut self, node: &LiteralExprNode) -> NewtResult {
        let literal = node.literal();
        let value = NewtValue::from_literal_node(node);

        Ok(value)
    }

    fn visit_grouping_expr(&mut self, node: &GroupingExprNode) -> NewtResult {
        let expr = node.expr();

        self.visit_expr(expr)
    }

    fn visit_variable_expr(&mut self, node: &VariableExprNode) -> NewtResult {
        match self.tree {
            Some(ref tree) => {
                let ref key: RefEquality<SyntaxNode> = node.to_inner().into();
                let offset = tree.resolutions().get(key).unwrap();
                self.scope
                    .resolve_at(*offset, node.identifier().lexeme())
                    .map(|value| value.clone())
            },
            None => panic!("We should always have a tree by now")
        }
    }

    fn visit_function_call_expr(&mut self, node: &FunctionCallExprNode) -> NewtResult {
        let callable = match self.visit_expr(node.callee())? {
            NewtValue::Callable(callable) => callable,
            _ => return Err(NewtRuntimeError::TypeError)
        };

        if callable.arity() != node.arguments().count() {
            return Err(NewtRuntimeError::TypeError);
        }

        let mut arguments: Vec<NewtValue> = Vec::new();
        for argument in node.arguments() {
            arguments.push(self.visit_expr(argument)?);
        }

        callable.call(self, &arguments)
    }
}

impl StmtVisitor<Result<(), NewtRuntimeError>> for VirtualMachineState {
    fn visit_stmt(&mut self, node: &StmtNode) -> Result<(), NewtRuntimeError> {
        if let Some(error) = self.halting_error {
            return Err(error);
        }

        let outcome = match node.kind() {
            StmtKind::VariableDeclarationStmt(node) => self.visit_variable_declaration_stmt(node),
            StmtKind::VariableAssignmentStmt(node) => self.visit_variable_assignment_stmt(node),
            StmtKind::StmtListStmt(node) => self.visit_stmt_list_stmt(node),
            StmtKind::ExprStmt(node) => self.visit_expr_stmt(node),
            StmtKind::IfStmt(node) => self.visit_if_stmt(node),
            StmtKind::WhileStmt(node) => self.visit_while_stmt(node),
            StmtKind::FunctionDeclarationStmt(node) => self.visit_function_declaration_stmt(node),
            StmtKind::ReturnStmt(node) => self.visit_return_stmt(node),
        };

        self.halt_on_error(outcome)?;

        outcome
    }

    fn visit_variable_declaration_stmt(
        &mut self,
        node: &VariableDeclarationStmtNode,
    ) -> Result<(), NewtRuntimeError> {
        let result = self.visit_expr(node.expr())?;
        let identifier = node.identifier().lexeme();
        let value = self.visit_expr(node.expr())?;

        match self.scope.bind(&identifier, value) {
            Err(error) => self.halt(error)?,
            Ok(_) => {}
        };

        Ok(())
    }

    fn visit_variable_assignment_stmt(
        &mut self,
        node: &VariableAssignmentStmtNode,
    ) -> Result<(), NewtRuntimeError> {
        let identifier = node.identifier().lexeme();
        let value = self.visit_expr(node.expr())?;

        self.scope.assign(identifier, value)
    }

    fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode) -> Result<(), NewtRuntimeError> {
        self.scope.push();

        for stmt in node.stmts() {
            self.visit_stmt(stmt)?;
        }

        self.scope.pop();

        Ok(())
    }

    fn visit_expr_stmt(&mut self, node: &ExprStmtNode) -> Result<(), NewtRuntimeError> {
        self.visit_expr(node.expr())?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, node: &IfStmtNode) -> Result<(), NewtRuntimeError> {
        let result = self.visit_expr(node.condition())?;

        match result {
            NewtValue::Bool(conditional) => {
                if conditional {
                    self.visit_stmt_list_stmt(node.when_true());
                } else {
                    if let Some(else_path) = node.when_false() {
                        self.visit_stmt_list_stmt(else_path);
                    }
                }
            }
            _ => self.halt(NewtRuntimeError::TypeError)?,
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, node: &WhileStmtNode) -> Result<(), NewtRuntimeError> {
        loop {
            let conditional = self.visit_expr(node.condition())?;
            let truthy_conditional = conditional.as_truthy().ok_or(NewtRuntimeError::TypeError)?;

            if !truthy_conditional {
                break;
            } else {
                self.visit_stmt_list_stmt(node.stmts())?;
            }
        }

        Ok(())
    }

    fn visit_function_declaration_stmt(
        &mut self,
        node: &FunctionDeclarationStmtNode,
    ) -> Result<(), NewtRuntimeError> {
        unimplemented!()
    }

    fn visit_return_stmt(&mut self, node: &ReturnStmtNode) -> Result<(), NewtRuntimeError> {
        unimplemented!()
    }
}

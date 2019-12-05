use crate::featurez::syntax::*;
use crate::featurez::TokenKind;
use std::collections::HashMap;
use crate::featurez::runtime::{Callable};
use crate::featurez::newtypes::TransparentNewType;
use crate::featurez::runtime::scope::{ScopeNode, Environment};
use crate::featurez::runtime::callable::NewtCallable;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct VirtualMachine {
    scope: Environment
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine {
            scope: Environment::new()
        }
    }

	pub fn new_with_scope(scope: &Environment) -> VirtualMachine {
		VirtualMachine {
			scope: scope.clone()
		}
	}

    pub fn interpret<S: Into<SyntaxTree>>(&mut self, source: S) -> NewtResult {
        let tree: SyntaxTree = source.into();

        if tree.errors().count() != 0 {
            return Err(NewtRuntimeError::InvalidSyntaxTree);
        }

        let node = tree.root()
            .as_node()
            .ok_or(NewtRuntimeError::InvalidSyntaxTree)?;

        let result = if let Some(expr) = ExprNode::cast(node) {
            self.visit_expr(expr)
        } else if let Some(stmt) = StmtNode::cast(node) {
            self.visit_stmt(stmt)
                .map(|f| NewtValue::Null)
        } else {
            panic!("All nodes should be either an Expression or Statement!");
        };

        match result {
            Ok(value) => Ok(value),
            Err(NewtRuntimeError::ReturnedValue(value)) => Ok(value),
            Err(error) => Err(error)
        }
    }
}

impl ExprVisitor<NewtResult> for VirtualMachine {
    fn visit_expr(&mut self, node: &ExprNode) -> NewtResult {
        match node.kind() {
            ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
            ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
            ExprKind::PrimitiveLiteralExpr(node) => self.visit_primitive_literal_expr(node),
            ExprKind::ObjectLiteralExpr(node) => self.visit_object_literal_expr(node),
            ExprKind::ObjectPropertyExpr(node) => self.visit_object_property_expr(node),
            ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
            ExprKind::VariableExpr(node) => self.visit_variable_expr(node),
            ExprKind::FunctionCallExpr(node) => self.visit_function_call_expr(node),
        }
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
            kind => unreachable!("TokenKind {:?} is not a binary", kind),
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

    fn visit_primitive_literal_expr(&mut self, node: &PrimitiveLiteralExprNode) -> NewtResult {
        let literal = node.literal();
        let value = NewtValue::from_primitive_literal_node(node);

        Ok(value)
    }

    fn visit_grouping_expr(&mut self, node: &GroupingExprNode) -> NewtResult {
        let expr = node.expr();

        self.visit_expr(expr)
    }

    fn visit_variable_expr(&mut self, node: &VariableExprNode) -> NewtResult {
        self.scope
            .resolve(node.identifier().lexeme())
            .map(|value| value.clone())
    }

    fn visit_function_call_expr(&mut self, node: &FunctionCallExprNode) -> NewtResult {
        let callable = match self.visit_expr(node.callee())? {
            NewtValue::Callable(callable) => callable,
            _ => {
                return Err(NewtRuntimeError::TypeError);
            }
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

    fn visit_object_literal_expr(&mut self, node: &ObjectLiteralExprNode) -> Result<NewtValue, NewtRuntimeError> {
        let mut map: HashMap<String, NewtValue> = HashMap::new();

        for pair in node.fields().iter() {
            map.insert(pair.0.clone(), self.visit_expr(pair.1)?);
        }

        Ok(NewtValue::Object(Rc::new(RefCell::new(map))))
    }

    fn visit_object_property_expr(&mut self, node: &ObjectPropertyExprNode) -> Result<NewtValue, NewtRuntimeError> {
        match self.visit_expr(node.source_expr())? {
            NewtValue::Object(map) => map.borrow().get(node.identifier().lexeme())
                .map(|reference| reference.clone())
                .ok_or(NewtRuntimeError::UndefinedVariable),
            _ => Err(NewtRuntimeError::TypeError)
        }
    }
}

impl StmtVisitor<Result<(), NewtRuntimeError>> for VirtualMachine {
    fn visit_stmt(&mut self, node: &StmtNode) -> Result<(), NewtRuntimeError> {
        match node.kind() {
            StmtKind::VariableDeclarationStmt(node) => self.visit_variable_declaration_stmt(node)?,
            StmtKind::AssignmentStmt(node) => self.visit_assignment_stmt(node)?,
            StmtKind::StmtListStmt(node) => self.visit_stmt_list_stmt(node)?,
            StmtKind::ExprStmt(node) => self.visit_expr_stmt(node)?,
            StmtKind::IfStmt(node) => self.visit_if_stmt(node)?,
            StmtKind::WhileStmt(node) => self.visit_while_stmt(node)?,
            StmtKind::FunctionDeclarationStmt(node) => self.visit_function_declaration_stmt(node)?,
            StmtKind::ReturnStmt(node) => self.visit_return_stmt(node)?,
        };

        Ok(())
    }

    fn visit_variable_declaration_stmt(
        &mut self,
        node: &VariableDeclarationStmtNode,
    ) -> Result<(), NewtRuntimeError> {
        let result = self.visit_expr(node.expr())?;
        let identifier = node.identifier().lexeme();
        let value = self.visit_expr(node.expr())?;

        self.scope.bind(&identifier, value)?;

        Ok(())
    }

    fn visit_assignment_stmt(
        &mut self,
        node: &AssignmentStmtNode,
    ) -> Result<(), NewtRuntimeError> {
        let value = self.visit_expr(node.expr())?;

        match node.rval().kind() {
            RValKind::VariableRVal(variable) => {
                self.scope.assign(variable.identifier().lexeme(), value)
            }
            RValKind::ObjectPropertyRVal(property) => {
                let mut destination = self.visit_expr(property.source_expr())?;
                match destination {
                    NewtValue::Object(object) => {
                        object.borrow_mut().insert(property.identifier().lexeme().to_string(), value);
                        Ok(())
                    }
                    _ => Err(NewtRuntimeError::TypeError)
                }
            }
        }
    }

    fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode) -> Result<(), NewtRuntimeError> {
	    if node.has_braces() {
		    self.scope.push_scope();
	    }

        for stmt in node.stmts() {
            self.visit_stmt(stmt)?;
        }

        if node.has_braces() {
	        self.scope.pop_scope();
        }

        Ok(())
    }

    fn visit_expr_stmt(&mut self, node: &ExprStmtNode) -> Result<(), NewtRuntimeError> {
        self.visit_expr(node.expr())?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, node: &IfStmtNode) -> Result<(), NewtRuntimeError> {
        let result = self.visit_expr(node.condition())?;
        let truthiness = result.as_truthy();

        match truthiness {
            Some(conditional) => {
                if conditional {
                    self.visit_stmt_list_stmt(node.when_true())?;
                } else {
                    if let Some(else_path) = node.when_false() {
                        self.visit_stmt_list_stmt(else_path)?;
                    }
                }
            }
            None => Err(NewtRuntimeError::TypeError)?,
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
        let callable = NewtCallable::new(node, &self.scope);
        self.scope.bind(node.identifier().lexeme(), NewtValue::Callable(Rc::new(callable)))?;

        Ok(())
    }

    fn visit_return_stmt(&mut self, node: &ReturnStmtNode) -> Result<(), NewtRuntimeError> {
        match node.result() {
            Some(expr) => {
                let ok_is_err = self.visit_expr(expr)?;
                Err(NewtRuntimeError::ReturnedValue(ok_is_err))
            },
            None => Err(NewtRuntimeError::NullValueEncountered)
        }
    }
}

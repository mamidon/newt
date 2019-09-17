use crate::featurez::TokenKind;
use crate::featurez::syntax::*;
use super::scope::Scope;

#[derive(Debug)]
pub struct VirtualMachine {
	scope: Scope,
	halting_error: Option<NewtRuntimeError>
}

impl VirtualMachine {
	pub fn new() -> VirtualMachine {
		VirtualMachine {
			scope: Scope::new(),
			halting_error: None
		}
	}
	
	fn halt(&mut self, error: NewtRuntimeError) {
		if !self.halted() {
			self.halting_error = Some(error)
		}
	}
	
	fn halted(&self) -> bool {
		self.halting_error.is_some()
	}
}

impl ExprVisitor for VirtualMachine {
	fn visit_binary_expr(&self, node: &BinaryExprNode) -> NewtResult {
		let lhs = self.visit_expr(node.lhs())?;
		let rhs = self.visit_expr(node.rhs())?;

		match node.operator() {
			TokenKind::Plus => lhs + rhs,
			TokenKind::Minus => lhs - rhs,
			TokenKind::Star => lhs * rhs,
			TokenKind::Slash => lhs / rhs,
			_ => unreachable!("not a binary")
		}
	}

	//noinspection RsTypeCheck -- faulty on the match statement
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> NewtResult {
		let rhs = self.visit_expr(node.rhs())?;

		match node.operator() {
			TokenKind::Bang => !rhs,
			TokenKind::Minus => -rhs,
			_ => unreachable!("not a unary")
		}
	}

	fn visit_literal_expr(&self, node: &LiteralExprNode) -> NewtResult {
		let literal = node.literal();
		let value = NewtValue::from_literal_node(node);

		Ok(value)
	}

	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> NewtResult {
		let expr = node.expr();

		self.visit_expr(expr)
	}

	fn visit_variable_expr(&self, node: &VariableExprNode) -> NewtResult {
		self.scope.resolve(node.identifier().lexeme())
			.map(|value| value.clone())
			.ok_or(NewtRuntimeError::UndefinedVariable)
	}
}

impl StmtVisitor for VirtualMachine {
	fn visit_variable_declaration_stmt(&mut self, node: &VariableDeclarationStmtNode) {
		let result = self.visit_expr(node.expr());
		let identifier = node.identifier().lexeme();
		
		match result {
			Ok(value) => self.scope.bind(&identifier, value),
			Err(error) => self.halt(error)
		}
	}

	fn visit_variable_assignment_stmt(&mut self, node: &VariableAssignmentStmtNode) {
		let identifier = node.identifier().lexeme();
		
		let result = self.scope.resolve(&identifier)
			.ok_or(NewtRuntimeError::UndefinedVariable)
			.and_then(|_| self.visit_expr(node.expr()));
		
		match result {
			Ok(value) => self.scope.bind(&identifier, value),
			Err(error) => self.halt(error)
		};
	}

	fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode) {
		for stmt in node.stmts() {
			if self.halted() {
				return;
			}
			
			self.visit_stmt(stmt)
		}
	}

	fn visit_expr_stmt(&mut self, node: &ExprStmtNode) {
		self.visit_expr(node.expr());
	}

	fn visit_if_stmt(&mut self, node: &IfStmtNode) {
		let result = self.visit_expr(node.condition());

		match result {
			Ok(NewtValue::Bool(conditional)) => {
				if conditional {
					self.visit_stmt_list_stmt(node.when_true());
				} else {
					if let Some(else_path) = node.when_false() {
						self.visit_stmt_list_stmt(else_path)
					}
				}
			},
			Ok(_) => self.halt(NewtRuntimeError::TypeError),
			Err(error) => self.halt(error)
		}
	}

	fn visit_while_stmt(&mut self, node: &WhileStmtNode) {
		loop {
			let result = self.visit_expr(node.condition());

			match result {
				Ok(NewtValue::Bool(conditional)) => {
					if !conditional {
						break;
					}

					self.visit_stmt_list_stmt(node.stmts());
				},
				Ok(NewtValue::Int(value)) => {
					if value == 0 {
						break;
					}

					self.visit_stmt_list_stmt(node.stmts());

				},
				Ok(value) => {
					self.halt(NewtRuntimeError::TypeError);
					break;
				},
				Err(error) => {
					self.halt(error);
					break;
				}
			}
		}


	}
}

use crate::featurez::TokenKind;
use crate::featurez::syntax::{
	ExprKind,
	ExprNode,
	GroupingExprNode,
	BinaryExprNode,
	UnaryExprNode,
	LiteralExprNode,
	NewtResult,
	NewtValue,
	NewtRuntimeError,
	ExprVisitor,
	StmtVisitor,
	VariableAssignmentStmtNode,
	VariableDeclarationStmtNode
};
use super::scope::Scope;

pub struct ExprVirtualMachine {
	scope: Scope,
	halting_error: Option<NewtRuntimeError>
}

impl ExprVirtualMachine {
	pub fn new() -> ExprVirtualMachine { 
		ExprVirtualMachine {
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

impl ExprVisitor for ExprVirtualMachine {
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
}

impl StmtVisitor for ExprVirtualMachine {
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
}

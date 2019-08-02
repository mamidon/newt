use crate::featurez::newtypes::TransparentNewType;
use crate::featurez::tokens::{
	Token, 
	TokenKind
};
use crate::featurez::syntax::{
	SyntaxNode,
	SyntaxKind,
	SyntaxToken,
	AstNode,
	ExprKind,
	StmtKind
};

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::rc::Rc;

#[repr(transparent)]
pub struct StmtNode(SyntaxNode);

unsafe impl TransparentNewType for StmtNode {
	type Inner = SyntaxNode;
}

impl AstNode for StmtNode {
	fn cast(node: &SyntaxNode) -> Option<&Self> {
		match node.kind() {
			SyntaxKind::VariableDeclarationStmt
			| SyntaxKind::VariableAssignmentStmt
			=> Some(StmtNode::from_inner(node)),
			_ => None
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		self.to_inner()
	}
}

impl StmtNode {
	pub fn kind(&self) -> StmtKind {
		match self.syntax().kind() {
			SyntaxKind::VariableDeclarationStmt =>
				StmtKind::VariableDeclarationStmt(VariableDeclarationStmtNode::from_inner(self.syntax())),
			SyntaxKind::VariableAssignmentStmt =>
				StmtKind::VariableAssignmentStmt(VariableAssignmentStmtNode::from_inner(self.syntax())),
			_ => unreachable!("StmtNode cannot be constructed from invalid SyntaxKind")
		}
	}
}

#[repr(transparent)]
pub struct VariableDeclarationStmtNode(SyntaxNode);

unsafe impl TransparentNewType for VariableDeclarationStmtNode {
	type Inner = SyntaxNode;
}

impl VariableDeclarationStmtNode {
	pub fn identifier(&self) -> &SyntaxToken {
		self.0.nth_token(0)
	}
	
	pub fn expr(&self) -> &ExprNode { 
		ExprNode::cast(self.0.nth_node(1))
			.expect("Expected an expression node in variable declaration statement")
	}
}

#[repr(transparent)]
pub struct VariableAssignmentStmtNode(SyntaxNode);

unsafe impl TransparentNewType for VariableAssignmentStmtNode {
	type Inner = SyntaxNode;
}

impl VariableAssignmentStmtNode {
	pub fn identifier(&self) -> &SyntaxToken {
		self.0.nth_token(0)
	}

	pub fn expr(&self) -> &ExprNode {
		ExprNode::cast(self.0.nth_node(0))
			.expect("Expected an expression node in variable declaration statement")
	}
}

#[repr(transparent)]
pub struct StmtListStmtNode(SyntaxNode);

unsafe impl TransparentNewType for StmtListStmtNode {
	type Inner = SyntaxNode;
}

impl StmtListStmtNode {
	pub fn stmts(&self) -> impl IntoIterator<Item=&StmtNode> {
		self.0.children()
			.iter()
			.filter_map(|n| n.as_node())
			.filter_map(StmtNode::cast)
	}
}

#[repr(transparent)]
pub struct ExprNode(SyntaxNode);

unsafe impl TransparentNewType for ExprNode {
	type Inner = SyntaxNode;
}


impl<'a> From<&'a BinaryExprNode> for &'a ExprNode {
	fn from(node: &'a BinaryExprNode) -> Self {
		ExprNode::from_inner(node.to_inner())
	}
}

impl AstNode for ExprNode {
	fn cast(node: &SyntaxNode) -> Option<&Self> {
		match node.kind() {
			SyntaxKind::BinaryExpr
			| SyntaxKind::UnaryExpr
			| SyntaxKind::LiteralExpr
			| SyntaxKind::GroupingExpr
			=> Some(ExprNode::from_inner(node)),
			_ => None
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		self.to_inner()
	}
}

impl ExprNode {
	pub fn kind(&self) -> ExprKind {
		match self.syntax().kind() {
			SyntaxKind::BinaryExpr => ExprKind::BinaryExpr(BinaryExprNode::from_inner(self.to_inner())),
			SyntaxKind::UnaryExpr => ExprKind::UnaryExpr(UnaryExprNode::from_inner(self.to_inner())),
			SyntaxKind::LiteralExpr => ExprKind::LiteralExpr(LiteralExprNode::from_inner(self.to_inner())),
			SyntaxKind::GroupingExpr => ExprKind::GroupingExpr(GroupingExprNode::from_inner(self.to_inner())),
			_ => unreachable!("ExprNode cannot be constructed from invalid SyntaxKind")
		}
	}
}

#[repr(transparent)]
pub struct LiteralExprNode(SyntaxNode);

unsafe impl TransparentNewType for LiteralExprNode {
	type Inner = SyntaxNode;
}

impl LiteralExprNode {
	pub fn literal(&self) -> &SyntaxToken {
		self.0.nth_token(0)
	}
}

#[repr(transparent)]
pub struct BinaryExprNode(SyntaxNode);

unsafe impl TransparentNewType for BinaryExprNode {
	type Inner = SyntaxNode;
}

impl BinaryExprNode {
	pub fn operator(&self) -> TokenKind {
		self.0.nth_token(0).token_kind()
	}

	pub fn lhs(&self) -> &ExprNode {
		ExprNode::cast(self.0.nth_node(0)).unwrap()
	}

	pub fn rhs(&self) -> &ExprNode {
		ExprNode::cast(self.0.nth_node(1)).unwrap()
	}
}

#[repr(transparent)]
pub struct UnaryExprNode(SyntaxNode);

unsafe impl TransparentNewType for UnaryExprNode {
	type Inner = SyntaxNode;
}

impl UnaryExprNode {
	pub fn operator(&self) -> TokenKind {
		self.0.nth_token(0).token_kind()
	}

	pub fn rhs(&self) -> &ExprNode {
		ExprNode::cast(self.0.nth_node(0)).unwrap()
	}
}

pub struct GroupingExprNode(SyntaxNode);

unsafe impl TransparentNewType for GroupingExprNode {
	type Inner = SyntaxNode;
}

impl GroupingExprNode {
	pub fn expr(&self) -> &ExprNode { 
		ExprNode::cast(self.0.nth_node(0)).unwrap()
	}
}
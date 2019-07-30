use crate::featurez::newtypes::TransparentNewType;
use crate::featurez::tokens::{
	Token, 
	TokenKind
};
use crate::featurez::syntax::{
	SyntaxNode,
	SyntaxKind,
	SyntaxToken,
	AstNode
};
use crate::featurez::syntax::ExprKind;

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::rc::Rc;

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
			_ => unreachable!("This shouldn't happen")
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
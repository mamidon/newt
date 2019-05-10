use super::newtypes::TransparentNewType;
use super::tokens::{Token, TokenKind};

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::rc::Rc;

mod syntax_element;
mod syntax_kind;
mod syntax_node;
mod syntax_token;
mod syntax_tree;
mod tests;
mod text_tree_sink;
mod token_source;
mod tree_sink;

pub use self::syntax_element::SyntaxElement;
pub use self::syntax_kind::SyntaxKind;
pub use self::syntax_node::SyntaxNode;
pub use self::syntax_token::SyntaxToken;
pub use self::syntax_tree::SyntaxTree;
pub use self::text_tree_sink::TextTreeSink;
pub use self::token_source::TokenSource;
pub use self::tree_sink::TreeSink;

pub trait AstNode {
	fn cast(node: &SyntaxNode) -> Option<&Self>;
	fn syntax(&self) -> &SyntaxNode;
}

pub enum ExprKind<'a> {
	BinaryExpr(&'a BinaryExprNode),
	UnaryExpr(&'a UnaryExprNode),
	LiteralExpr(&'a LiteralExprNode)
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

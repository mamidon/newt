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

#[repr(transparent)]
pub struct LiteralExprNode(SyntaxNode);

unsafe impl TransparentNewType for LiteralExprNode {
    type Inner = SyntaxNode;
}

#[repr(transparent)]
pub struct BinaryExprNode(SyntaxNode);

impl BinaryExprNode {
    pub fn lhs(&self) -> &BinaryExprNode {
        let lhs_node = self.0.nth_node_kind(0, SyntaxKind::LiteralExpr);
        BinaryExprNode::from_inner(lhs_node)
    }
}

unsafe impl TransparentNewType for BinaryExprNode {
    type Inner = SyntaxNode;
}

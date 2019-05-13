mod ast_node;
mod nodes;
mod syntax_element;
mod syntax_kind;
mod syntax_node;
mod syntax_token;
mod syntax_tree;
mod tests;
mod text_tree_sink;
mod token_source;
mod tree_sink;

pub use self::ast_node::AstNode;
pub use self::expr_kind::ExprKind;
pub use self::nodes::*;
pub use self::syntax_element::SyntaxElement;
pub use self::syntax_kind::SyntaxKind;
pub use self::syntax_node::SyntaxNode;
pub use self::syntax_token::SyntaxToken;
pub use self::syntax_tree::SyntaxTree;
pub use self::text_tree_sink::TextTreeSink;
pub use self::token_source::TokenSource;
pub use self::tree_sink::TreeSink;

mod expr_kind {
	use crate::featurez::syntax::nodes::*;
	
	pub enum ExprKind<'a> {
		BinaryExpr(&'a BinaryExprNode),
		UnaryExpr(&'a UnaryExprNode),
		LiteralExpr(&'a LiteralExprNode)
	}
}

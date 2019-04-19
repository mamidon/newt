use crate::featurez::syntax::{
	SyntaxNode,
	SyntaxToken
};

pub enum SyntaxElement {
	Node(SyntaxNode),
	Token(SyntaxToken)
}

impl SyntaxElement {
	pub fn is_node(&self) -> bool {
		match self {
			SyntaxElement::Node(_) => true,
			SyntaxElement::Token(_) => false
		}
	}

	pub fn as_node(&self) -> Option<&SyntaxNode> {
		match self {
			SyntaxElement::Node(n) => Some(n),
			SyntaxElement::Token(_) => None
		}
	}
}

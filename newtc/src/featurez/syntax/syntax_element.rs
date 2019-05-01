use crate::featurez::syntax::{SyntaxNode, SyntaxToken};
use crate::featurez::TokenKind;

pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

impl SyntaxElement {
    pub fn is_node(&self) -> bool {
        match self {
            SyntaxElement::Node(_) => true,
            SyntaxElement::Token(_) => false,
        }
    }
	
	pub fn is_token(&self) -> bool {
		!self.is_node()
	}
	
	pub fn is_trivia_token(&self, kind: TokenKind) -> bool {
		match self {
			SyntaxElement::Token(t) => t.token_kind().is_trivia(),
			_ => false
		}
	}

    pub fn as_node(&self) -> Option<&SyntaxNode> {
        match self {
            SyntaxElement::Node(n) => Some(n),
            SyntaxElement::Token(_) => None,
        }
    }
}

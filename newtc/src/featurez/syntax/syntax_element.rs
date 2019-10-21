use crate::featurez::syntax::{SyntaxNode, SyntaxToken, SyntaxInfo};
use crate::featurez::TokenKind;

#[derive(Debug, Clone)]
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(SyntaxToken),
    Info(SyntaxInfo)
}

impl SyntaxElement {
    pub fn is_node(&self) -> bool {
        match self {
            SyntaxElement::Node(_) => true,
            _ => false,
        }
    }

    pub fn is_token(&self) -> bool {
        match self {
            SyntaxElement::Token(_) => true,
            _ => false
        }
    }

    pub fn is_info(&self) -> bool {
        match self {
            SyntaxElement::Info(_) => true,
            _ => false
        }
    }

    pub fn is_trivia_token(&self, kind: TokenKind) -> bool {
        match self {
            SyntaxElement::Token(t) => t.token_kind().is_trivia(),
            _ => false,
        }
    }

    pub fn as_node(&self) -> Option<&SyntaxNode> {
        match self {
            SyntaxElement::Node(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_token(&self) -> Option<&SyntaxToken> {
        match self {
            SyntaxElement::Token(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_info(&self) -> Option<&SyntaxInfo> {
        match self {
            SyntaxElement::Info(i) => Some(i),
            _ => None
        }
    }
}

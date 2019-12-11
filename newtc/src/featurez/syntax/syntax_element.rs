use crate::featurez::syntax::{SyntaxNode, SyntaxToken};
use crate::featurez::TokenKind;
use std::fmt::{Display, Formatter, Error};

#[derive(Debug, Clone)]
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(SyntaxToken)
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
            SyntaxElement::Token(t) if !t.token_kind().is_trivia() => Some(t),
            _ => None,
        }
    }
}

impl Display for SyntaxElement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            SyntaxElement::Token(token) => write!(f, "{}", token),
            SyntaxElement::Node(node) => {
                write!(f, "({}", node.kind())?;
                for child in node.children() {
                    write!(f, " {}", child)?;
                }

                write!(f, ")")
            }
        }
    }
}

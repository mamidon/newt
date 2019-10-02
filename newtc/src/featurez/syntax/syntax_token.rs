use crate::featurez::tokens::TokenKind;

#[derive(Debug)]
pub struct SyntaxToken {
    token_kind: TokenKind,
    length: usize,
    lexeme: String,
}

impl SyntaxToken {
    pub fn new(token_kind: TokenKind, length: usize, lexeme: &str) -> SyntaxToken {
        SyntaxToken {
            token_kind,
            length,
            lexeme: lexeme.to_string(),
        }
    }

    pub fn token_kind(&self) -> TokenKind {
        self.token_kind
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme.as_str()
    }
}

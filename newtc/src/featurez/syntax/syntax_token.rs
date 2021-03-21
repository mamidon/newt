use crate::featurez::tokens::TokenKind;
use std::fmt::{Display, Error, Formatter};

#[derive(Debug, Clone)]
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
        &self.lexeme
    }

    fn escape_whitespace(text: &str) -> String {
        let escaped_text: String = text
            .chars()
            .map(|c| match c {
                '\t' => "\\t".to_string(),
                '\n' => "\\n".to_string(),
                ' ' => "\\s".to_string(),
                c => c.to_string(),
            })
            .collect();

        escaped_text
    }
}

impl Display for SyntaxToken {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "({} '{}')",
            self.token_kind,
            SyntaxToken::escape_whitespace(&self.lexeme)
        )
    }
}

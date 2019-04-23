use super::*;
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Copy, Clone)]
pub struct Token {
    token_kind: TokenKind,
    length: usize,
}

impl Token {
    pub(super) fn new(token_kind: TokenKind, length: usize) -> Token {
        Token {
            token_kind: token_kind,
            length,
        }
    }

    fn merge_as(token_kind: TokenKind, left: &Token, right: &Token) -> Token {
        Token {
            token_kind: token_kind,
            length: left.length + right.length,
        }
    }

    pub fn tomb_stone() -> Token {
        Token::new(TokenKind::TombStone, 0)
    }

    pub fn end_of_file() -> Token {
        Token::new(TokenKind::EndOfFile, 0)
    }

    pub fn token_kind(&self) -> TokenKind {
        self.token_kind
    }

    pub fn lexeme_length(&self) -> usize {
        self.length
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:?}[{}]", self.token_kind, self.length)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:?}[{}]", self.token_kind, self.length)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.token_kind == other.token_kind && self.length == other.length
    }
}

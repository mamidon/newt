use crate::featurez::syntax::TokenSource;
use crate::featurez::tokens::{Token, TokenKind};
use std::iter::Skip;
use std::slice::Iter;

#[derive(Clone)]
pub struct StrTokenSource {
    tokens: Vec<Token>,
    offsets: Vec<usize>,
}

impl StrTokenSource {
    pub fn new(raw_tokens: Vec<Token>) -> StrTokenSource {
        let mut tokens: Vec<Token> = vec![];
        let mut offsets: Vec<usize> = vec![];
        let mut length: usize = 0;

        for token in raw_tokens.iter() {
            tokens.push(*token);
            offsets.push(length);
            length += token.lexeme_length();
        }

        StrTokenSource { tokens, offsets }
    }
}

impl TokenSource for StrTokenSource {
    fn token(&self, index: usize) -> Token {
        if index >= self.tokens.len() {
            Token::end_of_file()
        } else {
            self.tokens[index]
        }
    }

    fn token_kind(&self, index: usize) -> TokenKind {
        if index >= self.tokens.len() {
            TokenKind::EndOfFile
        } else {
            self.tokens[index].token_kind()
        }
    }
}
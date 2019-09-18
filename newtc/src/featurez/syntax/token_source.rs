use crate::featurez::tokens::{Token, TokenKind};
use std::slice::Iter;
use std::iter::Skip;

pub trait TokenSource {
    fn token(&self, pos: usize) -> Token;
    fn token_kind(&self, pos: usize) -> TokenKind;
}

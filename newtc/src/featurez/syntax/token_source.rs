use crate::featurez::tokens::{Token, TokenKind};
use std::iter::Skip;
use std::slice::Iter;

pub trait TokenSource {
    fn token(&self, pos: usize) -> Token;
    fn token_kind(&self, pos: usize) -> TokenKind;
}

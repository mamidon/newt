use crate::featurez::tokens::{
	Token,
	TokenKind
};

pub trait TokenSource {
	fn token(&self, pos: usize) -> Token;
	fn token_kind(&self, pos: usize) -> TokenKind;

	fn token2(&self, pos: usize) -> Option<(Token, Token)>;
	fn token3(&self, pos: usize) -> Option<(Token, Token, Token)>;
}
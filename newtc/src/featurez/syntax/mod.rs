use super::tokens::{Token, TokenType};

pub trait TokenSource {
	fn token(&self, pos: usize) -> Token;
	fn token_type(&self, pos: usize) -> TokenType;

	fn token2(&self, pos: usize) -> Option<(Token, Token)>;
	fn token3(&self, pos: usize) -> Option<(Token, Token, Token)>;
	
	fn adjacent_predicate<P: Fn(Token) -> bool>(&self, pos: usize, predicate: P) -> bool;
}

use crate::featurez::tokens::TokenKind;

pub struct SyntaxToken {
	token_kind: TokenKind,
	length: usize
}

impl SyntaxToken {
	pub fn new(token_kind: TokenKind, length: usize) -> SyntaxToken {
		SyntaxToken {
			token_kind,
			length
		}
	}
	
	pub fn token_kind(&self) -> TokenKind {
		self.token_kind
	}
	
	pub fn length(&self) -> usize {
		self.length
	}
}
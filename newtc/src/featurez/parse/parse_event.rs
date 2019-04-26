use crate::featurez::tokens::{Token,TokenKind};
use crate::featurez::syntax::SyntaxKind;

#[derive(Debug)]
pub enum ParseEvent {
	Token { kind: TokenKind, length: usize },
	Trivia { kind: TokenKind, length: usize },
	BeginNode { kind: SyntaxKind },
	EndNode,
}

impl ParseEvent {
	pub fn tombstone() -> ParseEvent {
		ParseEvent::BeginNode {
			kind: SyntaxKind::TombStone,
		}
	}
}
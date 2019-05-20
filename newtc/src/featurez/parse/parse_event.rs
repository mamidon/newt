use crate::featurez::tokens::{Token,TokenKind};
use crate::featurez::syntax::SyntaxKind;

/*
I need to think about this model some more.  The parser needs to record 
when:
	a node begins & ends
	a node has a forward parent
	a node is a forward parent
	an empty slot for a node that was never closed
	a meaningful token
	irrelevant trivia
*/

#[derive(Debug, PartialOrd, PartialEq)]
pub enum ParseEvent {
	Token { kind: TokenKind, length: usize },
	Trivia { kind: TokenKind, length: usize },
	BeginNode { kind: SyntaxKind, is_forward_parent: bool, forward_parent_offset: Option<usize> },
	EndNode,
}

impl ParseEvent {
	pub fn tombstone() -> ParseEvent {
		ParseEvent::BeginNode { kind: SyntaxKind::TombStone, is_forward_parent: false, forward_parent_offset: None }
	}
}
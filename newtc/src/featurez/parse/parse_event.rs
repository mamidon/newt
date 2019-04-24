use crate::featurez::tokens::Token;
use crate::featurez::syntax::SyntaxKind;

#[derive(Debug)]
pub enum ParseEvent {
	Token { token: Token },
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
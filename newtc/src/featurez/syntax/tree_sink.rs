use crate::featurez::syntax::{
	SyntaxKind,
	SyntaxToken,
	SyntaxElement
};

pub trait TreeSink {
	fn begin_node(&mut self, kind: SyntaxKind, offset: usize);
	fn attach_token(&mut self, token: SyntaxToken);
	fn end_node(&mut self, offset: usize);

	fn end_tree(self) -> SyntaxElement;
}
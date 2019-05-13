use crate::featurez::syntax::SyntaxNode;

pub trait AstNode {
	fn cast(node: &SyntaxNode) -> Option<&Self>;
	fn syntax(&self) -> &SyntaxNode;
}	

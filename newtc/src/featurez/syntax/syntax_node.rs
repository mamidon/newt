use crate::featurez::syntax::{
	SyntaxKind,
	SyntaxElement
};

pub struct SyntaxNode {
	kind: SyntaxKind,
	length: usize,
	children: Box<[SyntaxElement]>
}

impl SyntaxNode {
	pub fn new(kind: SyntaxKind, length: usize, children: Box<[SyntaxElement]>) -> SyntaxNode {
		SyntaxNode {
			kind,
			length,
			children
		}
	}
	
	pub fn nth_node_kind(&self, n: usize, kind: SyntaxKind) -> &SyntaxNode {
		let node = self.children.iter()
			.filter(|c| SyntaxNode::node_predicate(c, kind))
			.nth(n)
			.unwrap();

		SyntaxNode::node_selecter(node)
	}

	fn node_predicate(node: &SyntaxElement, kind: SyntaxKind) -> bool {
		match node {
			SyntaxElement::Node(n) => {
				n.kind == kind
			},
			_ => false
		}
	}

	fn node_selecter(node: &SyntaxElement) -> &SyntaxNode {
		match node {
			SyntaxElement::Node(n) => n,
			_ => panic!("noo")
		}
	}
	
	pub fn kind(&self) -> SyntaxKind { self.kind }
	pub fn length(&self) -> usize { self.length }
	pub fn children(&self) -> &[SyntaxElement] { &self.children }
}
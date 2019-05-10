use crate::featurez::syntax::{SyntaxElement, SyntaxToken, SyntaxKind};
use crate::featurez::TokenKind;

pub struct SyntaxNode {
    kind: SyntaxKind,
    length: usize,
    children: Box<[SyntaxElement]>,
}

impl SyntaxNode {
    pub fn new(kind: SyntaxKind, length: usize, children: Box<[SyntaxElement]>) -> SyntaxNode {
        SyntaxNode {
            kind,
            length,
            children,
        }
    }

    pub fn nth_node(&self, n: usize) -> &SyntaxNode {
        let node = self
            .children
            .iter()
			.filter_map(|e| e.as_node())
            .nth(n)
            .unwrap();

		node
    }

	pub fn nth_token(&self, n: usize) -> &SyntaxToken {
		let token = self
			.children
			.iter()
			.filter_map(|e| e.as_token())
			.nth(n)
			.unwrap();

		token
	}

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }
    pub fn length(&self) -> usize {
        self.length
    }
    pub fn children(&self) -> &[SyntaxElement] {
        &self.children
    }
}

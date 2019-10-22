use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxToken, SyntaxInfo};
use crate::featurez::TokenKind;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct SyntaxNode {
    kind: SyntaxKind,
    length: usize,
    children: Rc<[SyntaxElement]>,
}

impl SyntaxNode {
    pub fn new(kind: SyntaxKind, length: usize, children: Vec<SyntaxElement>) -> SyntaxNode {
        SyntaxNode {
            kind,
            length,
            children: children.into(),
        }
    }

    pub fn with_info(&/*mut*/ self, info: SyntaxInfo) {
        assert_eq!(1, Rc::strong_count(&self.children));

        let mut next_children = self.children.to_vec();
        next_children.push(SyntaxElement::Info(info));
        unimplemented!();
        //self.children = Rc::new(next_children.into());
    }

    pub fn nth_node(&self, n: usize) -> &SyntaxNode {
        self.try_nth_node(n).unwrap()
    }

    pub fn try_nth_node(&self, n: usize) -> Option<&SyntaxNode> {
        let node = self.children.iter().filter_map(|e| e.as_node()).nth(n);

        node
    }

    pub fn nodes(&self) -> impl Iterator<Item = &SyntaxNode> {
        self.children.iter().filter_map(|e| e.as_node())
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

    pub fn infos(&self) -> impl Iterator<Item = &SyntaxInfo> {
        self.children.iter().filter_map(|e| e.as_info())
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }
    pub fn length(&self) -> usize {
        self.length
    }
    pub fn children(&self) -> &[SyntaxElement] {
        &*self.children
    }
}

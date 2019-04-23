use crate::featurez::syntax::SyntaxElement;
use crate::featurez::syntax::SyntaxKind;
use crate::featurez::syntax::SyntaxNode;
use crate::featurez::syntax::SyntaxToken;
use crate::featurez::syntax::TreeSink;

pub struct TextTreeSink {
    stack: Vec<(SyntaxKind, usize, usize)>,
    working_set: Vec<SyntaxElement>,
}

impl TextTreeSink {
    pub fn new() -> TextTreeSink {
        TextTreeSink {
            stack: vec![],
            working_set: vec![],
        }
    }
}

impl TreeSink for TextTreeSink {
    fn begin_node(&mut self, kind: SyntaxKind, offset: usize) {
        self.stack.push((kind, self.working_set.len(), offset));
    }

    fn attach_token(&mut self, token: SyntaxToken) {
        self.working_set.push(SyntaxElement::Token(token));
    }

    fn end_node(&mut self, offset: usize) {
        let (kind, children_start, offset_start) = self.stack.pop().unwrap();
        let mut children: Vec<SyntaxElement> = vec![];

        while self.working_set.len() > children_start {
            children.push(self.working_set.pop().unwrap())
        }
        children.reverse();

        let node = SyntaxNode::new(kind, offset - offset_start, children.into_boxed_slice());

        self.working_set.push(SyntaxElement::Node(node));
    }

    fn end_tree(mut self) -> SyntaxElement {
        self.working_set.remove(0)
    }
}

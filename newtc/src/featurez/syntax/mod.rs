use super::tokens::{Token, TokenType};
use std::rc::Rc;

#[derive(PartialOrd, PartialEq, Debug)]
pub enum SyntaxKind {
	BinaryExpr,
	LiteralExpr
}

pub struct SyntaxNode {
	kind: SyntaxKind,
	children: Box<[SyntaxElement]>
}

pub struct SyntaxToken {
	token_type: TokenType,
}

pub enum SyntaxElement {
	Node(SyntaxNode),
	Token(SyntaxToken)
}

pub trait TokenSource {
	fn token(&self, pos: usize) -> Token;
	fn token_type(&self, pos: usize) -> TokenType;

	fn token2(&self, pos: usize) -> Option<(Token, Token)>;
	fn token3(&self, pos: usize) -> Option<(Token, Token, Token)>;
}

pub trait TreeSink {
	fn begin_node(&mut self, kind: SyntaxKind);
	fn attach_token(&mut self, token: SyntaxToken);
	fn end_node(&mut self);
	
	fn end_tree(self) -> SyntaxElement;
}

struct TextTreeSink {
	stack: Vec<(SyntaxKind, usize)>,
	working_set: Vec<SyntaxElement>
}

impl TextTreeSink {
	pub fn new() -> TextTreeSink {
		TextTreeSink {
			stack: vec![],
			working_set: vec![]
		}
	}
}

/*
Before proceeding further, I need proper data structures.

SyntaxKind = distinguish between parse tree node types
SyntaxNode = (SyntaxKind, [children]) = non-leaf node in the parse tree
SyntaxToken = (TokenType) = leaf node in the parse tree
SyntaxElement = SyntaxToken | SyntaxNode = enum of either a leaf or non-leaf node

*/

impl TreeSink for TextTreeSink {
	fn begin_node(&mut self, kind: SyntaxKind) {
		self.stack.push((kind, self.working_set.len()));
	}

	fn attach_token(&mut self, token: SyntaxToken) {
		self.working_set.push(SyntaxElement::Token(token));
	}

	fn end_node(&mut self) {
		let node_meta = self.stack.pop().unwrap();
		let mut children: Vec<SyntaxElement> = vec![];
		
		while self.working_set.len() > node_meta.1 {
			children.push(self.working_set.pop().unwrap())
		}
		children.reverse();
		
		let node = SyntaxNode {
			kind: node_meta.0,
			children: children.into_boxed_slice(),
		};
		
		self.working_set.push(SyntaxElement::Node(node));
	}
	
	fn end_tree(mut self) -> SyntaxElement {
		self.working_set.remove(0)
	}
}

pub struct Parser<'a> {
	source: &'a TokenSource,
	consumed: usize,
}
/*
mod tests {
	use crate::featurez::tokens::{tokenize, StrTokenSource};
	use super::{TextTreeSink, TokenSource, SyntaxKind};
	use crate::featurez::syntax::TreeSink;
	use crate::featurez::syntax::SyntaxToken;
	use crate::featurez::tokens::TokenType;
	use crate::featurez::syntax::SyntaxElement;
	
	#[test]
	fn parse_from_tokens() {
		let text = "2+a";
		let tokens = tokenize(text);
		let source = StrTokenSource::new(text, tokens);
		let mut sink = TextTreeSink::new();
		
		sink.begin_node(SyntaxKind::BinaryExpr);
		sink.attach_token(SyntaxToken {token_type: TokenType::Identifier });
		sink.attach_token(SyntaxToken {token_type: TokenType::Plus });
	
			sink.begin_node(SyntaxKind::BinaryExpr);
				sink.attach_token(SyntaxToken {token_type: TokenType::Identifier });
				sink.attach_token(SyntaxToken {token_type: TokenType::Plus });
				sink.attach_token(SyntaxToken {token_type: TokenType::Identifier });
			sink.end_node();
		sink.end_node();
		
		print_tree(&sink.end_tree());
	}
	
	fn print_tree(root: &SyntaxElement) {
		print_tree_element(&root, 0);
	}
	
	fn print_tree_element(element: &SyntaxElement, depth: usize) {
		let prefix = "-".repeat(depth);
		
		match element {
			SyntaxElement::Node(node) => {
				println!("{}{:?}", prefix, node.kind);
				
				for child in node.children.iter() {
					print_tree_element(child, depth+1);
				}
			},
			SyntaxElement::Token(token) => {
				println!("{}{:?}", prefix, token.token_type);
			}
		}
	}
}
*/


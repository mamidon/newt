use super::tokens::{Token, TokenKind};
use std::rc::Rc;
use super::newtypes::TransparentNewType;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum SyntaxKind {
	BinaryExpr,
	LiteralExpr
}

pub struct SyntaxNode {
	kind: SyntaxKind,
	length: usize,
	children: Box<[SyntaxElement]>
}

impl SyntaxNode {
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
}

pub struct SyntaxToken {
	token_kind: TokenKind,
	length: usize
}

pub enum SyntaxElement {
	Node(SyntaxNode),
	Token(SyntaxToken)
}

impl SyntaxElement {
	pub fn is_node(&self) -> bool {
		match self {
			SyntaxElement::Node(_) => true,
			SyntaxElement::Token(_) => false
		}
	}
	
	pub fn as_node(&self) -> Option<&SyntaxNode> {
		match self {
			SyntaxElement::Node(n) => Some(n),
			SyntaxElement::Token(_) => None
		}
	}
}

pub struct SyntaxTree<'a> {
	text: &'a str,
	root: &'a SyntaxElement
}

impl<'a> SyntaxTree<'a> {
	pub fn new(root: &'a SyntaxElement, text: &'a str) -> SyntaxTree<'a> {
		SyntaxTree {
			text,
			root
		}
	}
}

#[repr(transparent)]
pub struct LiteralExprNode(SyntaxNode);

unsafe impl TransparentNewType for LiteralExprNode {
	type Inner = SyntaxNode;
}

#[repr(transparent)]
pub struct BinaryExprNode(SyntaxNode);

impl BinaryExprNode {
	pub fn lhs(&self) -> &BinaryExprNode {
		let lhs_node = self.0.nth_node_kind(0, SyntaxKind::LiteralExpr);
		BinaryExprNode::from_inner(lhs_node)
	}
}

unsafe impl TransparentNewType for BinaryExprNode {
	type Inner = SyntaxNode;
}

pub trait TokenSource {
	fn token(&self, pos: usize) -> Token;
	fn token_kind(&self, pos: usize) -> TokenKind;

	fn token2(&self, pos: usize) -> Option<(Token, Token)>;
	fn token3(&self, pos: usize) -> Option<(Token, Token, Token)>;
}

pub trait TreeSink {
	fn begin_node(&mut self, kind: SyntaxKind, offset: usize);
	fn attach_token(&mut self, token: SyntaxToken);
	fn end_node(&mut self, offset: usize);
	
	fn end_tree(self) -> SyntaxElement;
}

struct TextTreeSink {
	stack: Vec<(SyntaxKind, usize, usize)>,
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

impl TreeSink for TextTreeSink {
	fn begin_node(&mut self, kind: SyntaxKind, offset: usize) {
		self.stack.push((kind, self.working_set.len(), offset));
	}

	fn attach_token(&mut self, token: SyntaxToken) {
		self.working_set.push(SyntaxElement::Token(token));
	}

	fn end_node(&mut self, offset: usize) {
		let (kind, children_start, offset_start) 
			= self.stack.pop().unwrap();
		let mut children: Vec<SyntaxElement> = vec![];
		
		while self.working_set.len() > children_start {
			children.push(self.working_set.pop().unwrap())
		}
		children.reverse();
		
		let node = SyntaxNode {
			kind,
			length: offset - offset_start,
			children: children.into_boxed_slice(),
		};
		
		self.working_set.push(SyntaxElement::Node(node));
	}
	
	fn end_tree(mut self) -> SyntaxElement {
		self.working_set.remove(0)
	}
}

impl<'a> Display for SyntaxTree<'a> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		print_tree_element(f, self.root, 0, 0, self.text);
		return Ok(());
			
		fn print_tree_element(f: &mut Formatter, element: &SyntaxElement, depth: usize, offset: usize, text: &str) -> usize {
			let prefix = "-".repeat(depth);

			match element {
				SyntaxElement::Node(node) => {
					writeln!(f, "[{}..{}) {}{:?} '{}'",
							 offset,
							 offset + node.length,
							 prefix,
							 node.kind,
							 &text[offset..offset+node.length]);

					let mut children_offset = offset;
					for child in node.children.iter() {
						children_offset += print_tree_element(f, child, depth+1, children_offset, text);
					}
					return children_offset;
				},
				SyntaxElement::Token(token) => {
					writeln!(f, "[{}..{}) {}{:?} '{}'",
							 offset,
							 offset + token.length,
							 prefix,
							 token.token_kind,
							 &text[offset..offset + token.length]);
					return token.length;
				}
			}
		}
	}
	
	
}

pub struct Parser<'a> {
	source: &'a TokenSource,
	consumed: usize,
}

mod tests {
	use crate::featurez::tokens::{tokenize, StrTokenSource};
	use super::{TextTreeSink, TokenSource, SyntaxKind};
	use super::TransparentNewType;
	use crate::featurez::syntax::TreeSink;
	use crate::featurez::syntax::SyntaxToken;
	use crate::featurez::tokens::TokenKind;
	use crate::featurez::syntax::SyntaxElement;
	use crate::featurez::syntax::BinaryExprNode;
	use crate::featurez::syntax::SyntaxTree;

	#[test]
	fn parse_from_tokens() {
		let text = "a+b+c";
		let tokens = tokenize(text);
		let source = StrTokenSource::new(text, tokens);
		let mut sink = TextTreeSink::new();
		
		sink.begin_node(SyntaxKind::BinaryExpr, 0);
		sink.attach_token(SyntaxToken {token_kind: TokenKind::Identifier, length: 1 });
		sink.attach_token(SyntaxToken {token_kind: TokenKind::Plus, length: 1 });
	
			sink.begin_node(SyntaxKind::BinaryExpr, 2);
				sink.attach_token(SyntaxToken {token_kind: TokenKind::Identifier, length: 1 });
				sink.attach_token(SyntaxToken {token_kind: TokenKind::Plus, length: 1 });
				sink.attach_token(SyntaxToken {token_kind: TokenKind::Identifier, length: 1 });
			sink.end_node(5);
		sink.end_node(5);
		
		let root= &sink.end_tree();
		let tree = SyntaxTree::new(&root, text);
		println!("{}", tree);
	}
}


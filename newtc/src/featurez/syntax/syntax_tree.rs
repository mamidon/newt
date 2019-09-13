use crate::featurez::syntax::SyntaxElement;
use crate::featurez::syntax::TextTreeSink;
use crate::featurez::syntax::tree_sink::TreeSink;
use crate::featurez::syntax::SyntaxToken;
use crate::featurez::parse::CompletedParsing;
use crate::featurez::parse::ParseEvent;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::iter::{Iterator, IntoIterator};

pub struct SyntaxTree<'a> {
    text: &'a str,
    root: SyntaxElement,
}

impl<'a> SyntaxTree<'a> {
    pub fn new(root: SyntaxElement, text: &'a str) -> SyntaxTree<'a> {
        SyntaxTree { text, root }
    }
	
	pub fn root(&self) -> &SyntaxElement { &self.root }
	
	pub fn from_parser(parser: &CompletedParsing, text: &'a str) -> Self {
		let events = &parser.events;
		let mut sink = TextTreeSink::new();
		let mut offset = 0;
		for (index, event) in events.iter().enumerate() {
			match event {
				ParseEvent::BeginNode { kind: k, is_forward_parent: false, forward_parent_offset } => {
					if let Some(first_parent_offset) = forward_parent_offset {
						let mut offset = *first_parent_offset;

						loop {
							match &events[index + offset] {
								ParseEvent::BeginNode {
									kind: parent_kind,
									is_forward_parent: true,
									forward_parent_offset: next_offset
								} => {
									sink.begin_node(*parent_kind, 0);
									if let Some(next_offset) = next_offset {
										offset += *next_offset
									} else {
										break
									}
								},
								_ => break
							};
						}
					}
					
					sink.begin_node(*k, 0);
				},
				ParseEvent::BeginNode { kind: _, is_forward_parent: true, forward_parent_offset: _ } => {
					// noop
				},
				ParseEvent::EndNode => {
					sink.end_node(0)
				},
				ParseEvent::Token { kind: k, length: l } => {
					sink.attach_token(SyntaxToken::new(*k, *l, &text[offset..offset + l]));
					offset += l;
				},
				ParseEvent::Trivia { kind: k, length: l } => {
					sink.attach_token(SyntaxToken::new(*k, *l, &text[offset..offset + l]));
					offset += *l;
				}
			}
		}

		let root = sink.end_tree();

		SyntaxTree::new(root, text)
	}

	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		print_tree_element(f, &self.root, "", self.text, true);
		return Ok(());

		fn print_tree_element(
			f: &mut Formatter,
			element: &SyntaxElement,
			prefix: &str,
			text: &str,
			last: bool
		) -> usize {
			write!(f, "{}", prefix);
			let next_prefix = if last {
				write!(f, "┗ ");
				prefix.to_owned() + "  "
			} else {
				write!(f, "┠ ");
				prefix.to_owned() + "┃ "
			};

			match element {
				SyntaxElement::Node(node) => {
					writeln!(f, "{:?}", node.kind());

					let mut children_length = 0;
					for (index, child) in node.children().iter().enumerate() {
						let last_child = node.children().len() - 1 == index;

						children_length += print_tree_element(f, child, &next_prefix, &text[children_length..], last_child);
					}
					return children_length;
				}
				SyntaxElement::Token(token) => {
					writeln!(f, "{:?} {:?}", token.token_kind(), &text[..token.length()]);

					return token.length();
				}
			}
		}
	}
}

impl<'a> Display for SyntaxTree<'a> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		SyntaxTree::fmt(self, f)
	}
}

impl<'a> Debug for SyntaxTree<'a> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		SyntaxTree::fmt(self, f)
	}
}

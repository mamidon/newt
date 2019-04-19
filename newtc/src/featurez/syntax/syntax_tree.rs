use crate::featurez::syntax::SyntaxElement;
use std::fmt::Formatter;
use std::fmt::Error;
use std::fmt::Display;

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
							 offset + node.length(),
							 prefix,
							 node.kind(),
							 &text[offset..offset+node.length()]);

					let mut children_offset = offset;
					for child in node.children().iter() {
						children_offset += print_tree_element(f, child, depth+1, children_offset, text);
					}
					return children_offset;
				},
				SyntaxElement::Token(token) => {
					writeln!(f, "[{}..{}) {}{:?} '{}'",
							 offset,
							 offset + token.length(),
							 prefix,
							 token.token_kind(),
							 &text[offset..offset + token.length()]);
					return token.length();
				}
			}
		}
	}
}
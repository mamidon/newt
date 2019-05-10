use crate::featurez::syntax::SyntaxElement;
use crate::featurez::syntax::TextTreeSink;
use crate::featurez::syntax::tree_sink::TreeSink;
use crate::featurez::syntax::SyntaxToken;
use crate::featurez::Parser;
use crate::featurez::parse::ParseEvent;

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

pub struct SyntaxTree<'a> {
    text: &'a str,
    root: SyntaxElement,
}

impl<'a> SyntaxTree<'a> {
    pub fn new(root: SyntaxElement, text: &'a str) -> SyntaxTree<'a> {
        SyntaxTree { text, root }
    }
	pub fn root(&self) -> &SyntaxElement { &self.root }
	pub fn from_parser(parser: Parser, text: &'a str) -> Self {
		let events = parser.end_parsing();
		let mut sink = TextTreeSink::new();
		let mut offset = 0;
		for event in events.into_iter() {
			match event {
				ParseEvent::BeginNode { kind: k } => {
					sink.begin_node(k, 0);
				},
				ParseEvent::EndNode => {
					sink.end_node(0)
				},
				ParseEvent::Token { kind: k, length: l } => {
					sink.attach_token(SyntaxToken::new(k, l, &text[offset..offset + l]));
					offset += l;
				},
				ParseEvent::Trivia { kind: k, length: l } => {
					sink.attach_token(SyntaxToken::new(k, l, &text[offset..offset + l]));
					offset += l;
				}
			}
		}

		let root = sink.end_tree();

		SyntaxTree::new(root, text)
	}
}

impl<'a> Display for SyntaxTree<'a> {
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

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

	pub fn from_parser(parser: Parser, text: &'a str) -> Self {
		let events = parser.end_parsing();
		let mut sink = TextTreeSink::new();

		for event in events.into_iter() {
			match event {
				ParseEvent::BeginNode { kind: k } => {
					println!("begin_node: {:?}", k);
					sink.begin_node(k, 0);
				},
				ParseEvent::EndNode => {
					println!("end_node");
					sink.end_node(0)
				},
				ParseEvent::Token { kind: k, length: l } => {
					println!("token: {:?}, length: {}", k, l);
					sink.attach_token(SyntaxToken::new(k, l))
				},
				ParseEvent::Trivia { kind: k, length: l } => {
					println!("trivia: {:?}, length: {}", k, l);
					sink.attach_token(SyntaxToken::new(k, l))
				}
			}
		}

		let root = sink.end_tree();

		SyntaxTree::new(root, text)
	}
}

impl<'a> Display for SyntaxTree<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        print_tree_element(f, &self.root, 0, 0, self.text);
        return Ok(());

        fn print_tree_element(
            f: &mut Formatter,
            element: &SyntaxElement,
            depth: usize,
            offset: usize,
            text: &str,
        ) -> usize {
            let prefix = "-".repeat(depth);

            match element {
                SyntaxElement::Node(node) => {
                    writeln!(
                        f,
                        "[{}..{}) {}{:?} '{}'",
                        offset,
                        offset + node.length(),
                        prefix,
                        node.kind(),
                        &text[offset..offset + node.length()]
                    );

                    let mut children_offset = offset;
                    for child in node.children().iter() {
                        children_offset +=
                            print_tree_element(f, child, depth + 1, children_offset, text);
                    }
                    return children_offset;
                }
                SyntaxElement::Token(token) => {
                    writeln!(
                        f,
                        "[{}..{}) {}{:?} '{}'",
                        offset,
                        offset + token.length(),
                        prefix,
                        token.token_kind(),
                        &text[offset..offset + token.length()]
                    );
                    return token.length();
                }
            }
        }
    }
}

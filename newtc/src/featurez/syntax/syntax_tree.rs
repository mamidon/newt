use crate::featurez::parse::{CompletedParsing, Parser};
use crate::featurez::parse::ParseEvent;
use crate::featurez::syntax::tree_sink::TreeSink;
use crate::featurez::syntax::{AstNode, SyntaxElement, SyntaxNode, StmtNode, SyntaxKind};
use crate::featurez::syntax::SyntaxToken;
use crate::featurez::syntax::TextTreeSink;
use crate::featurez::tokenize;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::collections::{HashSet, HashMap};
use crate::featurez::driver::NewtError;
use crate::featurez::{TokenKind, StrTokenSource};
use crate::featurez::grammar::{root_stmt, root_expr};

pub struct SyntaxTree {
    root: SyntaxElement,
    errors: Vec<ErrorReport>
}

pub struct ErrorReport {
    pub(crate) line: usize,
    pub(crate) message: String
}

impl SyntaxTree {
    fn new(root: SyntaxElement, errors: Vec<ErrorReport>) -> SyntaxTree {
        SyntaxTree {
            root,
            errors
        }
    }

    pub fn root(&self) -> &SyntaxElement {
        &self.root
    }

    pub fn from_parser(parser: &CompletedParsing, text: &str) -> Self {
        let events = &parser.events;

        let mut sink = TextTreeSink::new();
        let mut offset = 0;
        let mut error_reports: Vec<ErrorReport> = Vec::new();
        let mut lines = 0;

        for (index, event) in events.iter().enumerate() {
            match event {
                ParseEvent::BeginNode {
                    kind: k,
                    is_forward_parent: false,
                    forward_parent_offset,
                } => {
                    match k {
                        SyntaxKind::Error(message) => {
                            error_reports.push(ErrorReport {
                                message: message.to_string(),
                                line: lines + 1
                            })
                        },
                        _ => {}
                    }
                    Self::begin_forward_parents(&mut sink, &events, index);
                }
                ParseEvent::BeginNode {
                    kind,
                    is_forward_parent: true,
                    forward_parent_offset: _,
                } => {
                    match kind {
                        SyntaxKind::Error(message) => {
                            error_reports.push(ErrorReport {
                                message: message.to_string(),
                                line: lines + 1
                            })
                        },
                        _ => {}
                    }
                }
                ParseEvent::EndNode => {
                    sink.end_node(0);
                },
                ParseEvent::Token { kind: k, length: l } => {
                    sink.attach_token(SyntaxToken::new(*k, *l, &text[offset..offset + l]));
                    offset += l;
                }
                ParseEvent::Trivia { kind: k, length: l } => {
                    let lexeme = &text[offset..offset + l];
                    lines = lines + lexeme.chars().filter(|c| *c == '\n').count();

                    sink.attach_token(SyntaxToken::new(*k, *l, lexeme));
                    offset += *l;
                }
            }
        }

        let root = sink.end_tree();

        SyntaxTree::new(root, error_reports)
    }

    pub fn iter(&self) -> SyntaxTreeIterator {
        SyntaxTreeIterator {
            frontier: vec![self.root()],
        }
    }

    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        print_tree_element(f, &self.root, "",true);
        return Ok(());

        fn print_tree_element(
            f: &mut Formatter,
            element: &SyntaxElement,
            prefix: &str,
            last: bool,
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

                        children_length += print_tree_element(
                            f,
                            child,
                            &next_prefix,
                            last_child,
                        );
                    }
                    return children_length;
                }
                SyntaxElement::Token(token) => {
                    writeln!(f, "{:?} '{}'", token.token_kind(), token.lexeme());

                    return token.length();
                }
            }
        }
    }

    fn begin_forward_parents(sink: &mut TextTreeSink, events: &[ParseEvent], index: usize) {
        let event = &events[index];

        match event {
            ParseEvent::BeginNode {
                kind,
                forward_parent_offset: offset,
                is_forward_parent: _
            } => {
                if let Some(next_offset) = offset {
                    Self::begin_forward_parents(sink, events, index + *next_offset);
                }
                sink.begin_node(*kind, 0);
            },
            _ => {}
        }
    }
}

impl Display for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        SyntaxTree::fmt(self, f)
    }
}

impl Debug for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        SyntaxTree::fmt(self, f)
    }
}


pub struct SyntaxTreeIterator<'a> {
    frontier: Vec<&'a SyntaxElement>
}

impl<'a> Iterator for SyntaxTreeIterator<'a> {
    type Item = &'a SyntaxElement;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.frontier.pop();

        if let Some(SyntaxElement::Node(node)) = next {
            self.frontier.extend(node.children().iter().rev());
        }

        return next;
    }
}



impl From<&str> for SyntaxTree {
    fn from(source: &str) -> Self {
        let statement_token_kinds = [
            TokenKind::SemiColon,
            TokenKind::RightBrace,
            TokenKind::LeftBrace,
            TokenKind::RightBracket,
            TokenKind::LeftBracket
        ];
        let tokens = tokenize(source);
        let statement_tokens = tokens.iter().any(|t| statement_token_kinds.contains(&t.token_kind()));
        let token_source = StrTokenSource::new(tokens);
        let mut p = Parser::new(token_source);

        let parsing = if statement_tokens {
            root_stmt(p)
        } else {
            root_expr(p)
        };

        SyntaxTree::from_parser(&parsing, source)
    }
}


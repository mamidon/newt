use crate::featurez::parse::ParseEvent;
use crate::featurez::parse::{CompletedParsing, Parser};
use crate::featurez::syntax::tree_sink::TreeSink;
use crate::featurez::syntax::SyntaxToken;
use crate::featurez::syntax::TextTreeSink;
use crate::featurez::syntax::{AstNode, StmtNode, SyntaxElement, SyntaxKind, SyntaxNode};
use crate::featurez::tokenize;

use crate::featurez::driver::NewtError;
use crate::featurez::grammar::{root_expr, root_stmt};
use crate::featurez::{StrTokenSource, TokenKind};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

pub struct SyntaxTree {
    root: SyntaxElement,
    errors: Vec<ErrorReport>,
}

pub struct ErrorReport {
    pub(crate) line: usize,
    pub(crate) message: String,
}

impl Display for ErrorReport {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}: {}", self.line, self.message)
    }
}

impl SyntaxTree {
    fn new(root: SyntaxElement, errors: Vec<ErrorReport>) -> SyntaxTree {
        SyntaxTree { root, errors }
    }

    pub fn root(&self) -> &SyntaxElement {
        &self.root
    }

    pub fn errors(&self) -> impl Iterator<Item = &ErrorReport> {
        self.errors.iter()
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
                        SyntaxKind::Error(message) => error_reports.push(ErrorReport {
                            message: message.to_string(),
                            line: lines + 1,
                        }),
                        _ => {}
                    }
                    Self::begin_forward_parents(&mut sink, &events, index);
                }
                ParseEvent::BeginNode {
                    kind,
                    is_forward_parent: true,
                    forward_parent_offset: _,
                } => match kind {
                    SyntaxKind::Error(message) => error_reports.push(ErrorReport {
                        message: message.to_string(),
                        line: lines + 1,
                    }),
                    _ => {}
                },
                ParseEvent::EndNode => {
                    sink.end_node(0);
                }
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

    fn pretty_print_tree_elements(
        f: &mut Formatter,
        element: &SyntaxElement,
        parents_child_count: Option<usize>,
        prefix: &str,
    ) -> Result<(), Error> {
        match parents_child_count {
            None => {}
            Some(1) => write!(f, " ")?,
            Some(_) => write!(f, "{}", prefix)?,
        }

        let mut next_prefix = prefix.to_string();
        next_prefix.push_str("\t");

        match element {
            SyntaxElement::Token(token) => {
                write!(f, "{}", token)?;
            }
            SyntaxElement::Node(node) => {
                match node.children().len() {
                    0 => {
                        write!(f, "({})", node.kind())?;
                    }
                    1 => {
                        write!(f, "({}", node.kind())?;
                        SyntaxTree::pretty_print_tree_elements(
                            f,
                            &node.children()[0],
                            Some(1),
                            &next_prefix,
                        )?;
                        write!(f, ")")?;
                    }
                    child_count => {
                        writeln!(f, "({}", node.kind())?;
                        SyntaxTree::pretty_print_tree_elements(
                            f,
                            &node.children()[0],
                            Some(child_count),
                            &next_prefix,
                        )?;
                        for child in &node.children()[1..] {
                            writeln!(f)?;
                            SyntaxTree::pretty_print_tree_elements(
                                f,
                                &child,
                                Some(child_count),
                                &next_prefix,
                            )?;
                        }
                        write!(f, ")")?;
                    }
                };
            }
        };

        Ok(())
    }

    fn begin_forward_parents(sink: &mut TextTreeSink, events: &[ParseEvent], index: usize) {
        let event = &events[index];

        match event {
            ParseEvent::BeginNode {
                kind,
                forward_parent_offset: offset,
                is_forward_parent: _,
            } => {
                if let Some(next_offset) = offset {
                    Self::begin_forward_parents(sink, events, index + *next_offset);
                }
                sink.begin_node(*kind, 0);
            }
            _ => {}
        }
    }
}

impl Display for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        SyntaxTree::pretty_print_tree_elements(f, &self.root, None, "")?;
        Ok(())
    }
}

impl Debug for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        SyntaxTree::pretty_print_tree_elements(f, &self.root, None, "")?;
        Ok(())
    }
}

pub struct SyntaxTreeIterator<'a> {
    frontier: Vec<&'a SyntaxElement>,
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
            TokenKind::LeftBracket,
        ];
        let tokens = tokenize(source);
        let statement_tokens = tokens
            .iter()
            .any(|t| statement_token_kinds.contains(&t.token_kind()));
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

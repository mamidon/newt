use crate::featurez::parse::marker::{CompletedMarker, Marker};
use crate::featurez::parse::ParseEvent;
use crate::featurez::syntax::{SyntaxKind, TokenSource};
use crate::featurez::tokens::TokenKind;
use crate::featurez::StrTokenSource;

use std::mem::replace;

#[derive(Clone)]
pub struct Parser {
    source: StrTokenSource,
    consumed_tokens: usize,
    events: Vec<ParseEvent>,
    panicking: bool,
    root_marker: Option<Marker>,
}

#[derive(Debug)]
pub struct CompletedParsing {
    pub events: Box<[ParseEvent]>,
    pub consumed_tokens: usize,
}

impl Parser {
    pub fn new(source: StrTokenSource) -> Parser {
        let mut p = Parser {
            source,
            consumed_tokens: 0,
            events: vec![],
            panicking: false,
            root_marker: None,
        };

        // begin implicit root node
        p.root_marker = Some(p.begin_node());
        p.eat_trivia();

        p
    }

    pub fn current(&self) -> TokenKind {
        self.source.token(self.consumed_tokens).token_kind()
    }

    pub fn current2(&self) -> Option<(TokenKind, TokenKind)> {
        let current0 = self.source.token(self.consumed_tokens).token_kind();
        let mut offset = 1;
        let mut current1 = self.source.token(self.consumed_tokens + offset);

        while current1.token_kind().is_trivia() && current1.token_kind() != TokenKind::EndOfFile {
            offset = offset + 1;
            current1 = self.source.token(self.consumed_tokens + offset);
        }

        Some((current0, current1.token_kind()))
    }

    pub fn nth(&self, n: usize) -> TokenKind {
        self.source.token(self.consumed_tokens + n).token_kind()
    }

    pub fn token(&mut self, kind: TokenKind) {
        if !self.token_if(kind) {
            panic!(
                "We assumed a token kind of {:?} but found {:?} instead",
                kind,
                self.current()
            )
        }
    }

    pub fn token_if(&mut self, kind: TokenKind) -> bool {
        if self.current() != kind {
            return false;
        }

        let token = self.source.token(self.consumed_tokens);
        self.consumed_tokens += 1;
        self.events.push(ParseEvent::Token {
            kind: token.token_kind(),
            length: token.lexeme_length(),
        });

        self.eat_trivia();

        return true;
    }

    pub fn expect_token_kind(&mut self, kind: TokenKind, msg: &'static str) {
        if self.token_if(kind) {
            self.panicking = false;
            return;
        }

        if self.panicking {
            self.remap_token(TokenKind::TombStone);
        } else {
            self.panicking = true;

            self.report_error(msg);
        }
    }

    pub fn expect_token_kind_in(&mut self, kinds: &[TokenKind], msg: &'static str) {
        for kind in kinds {
            if self.token_if(*kind) {
                self.panicking = false;
                return;
            }
        }

        if self.panicking {
            self.remap_token(TokenKind::TombStone);
        } else {
            self.panicking = true;

            self.report_error(msg);
        }
    }

    fn report_error(&mut self, message: &'static str) {
        let mut error = self.begin_node();

        let token = self.source.token(self.consumed_tokens);

        self.consumed_tokens += 1;
        self.events.push(ParseEvent::Token {
            kind: token.token_kind(),
            length: token.lexeme_length(),
        });

        self.eat_trivia();

        self.end_node(error, SyntaxKind::Error(message));
    }

    pub fn begin_node(&mut self) -> Marker {
        match self.get_implicit_root_node() {
            Some(marker) => marker,
            None => {
                let index = self.events.len();
                self.events.push(ParseEvent::tombstone());

                Marker::new(index)
            }
        }
    }

    fn get_implicit_root_node(&mut self) -> Option<Marker> {
        self.root_marker.take()
    }

    pub fn end_node(&mut self, marker: Marker, kind: SyntaxKind) -> CompletedMarker {
        let begin = &mut self.events[marker.index()];

        return match begin {
            ParseEvent::BeginNode {
                kind: ref mut slot,
                is_forward_parent: _,
                forward_parent_offset: _,
            } => {
                *slot = kind;
                let completed_marker = marker.defuse(self.events.len(), kind);
                self.events.push(ParseEvent::EndNode);

                return completed_marker;
            }
            _ => panic!(
                "Did not expect to complete a marker we don't have access to anymore!{:?}{}",
                begin,
                marker.index()
            ),
        };
    }

    pub fn precede_node(&mut self, child: &mut CompletedMarker, parent: &Marker) {
        match self.events[child.start()] {
            ParseEvent::BeginNode {
                kind: _,
                ref mut is_forward_parent,
                ref mut forward_parent_offset,
            } => {
                *forward_parent_offset = Some(parent.index() - child.start());
            }
            _ => panic!(
                "Expected BeginNode event, got {:?} event",
                &self.events[child.start()]
            ),
        }

        replace(
            &mut self.events[parent.index()],
            ParseEvent::BeginNode {
                kind: SyntaxKind::TombStone,
                is_forward_parent: true,
                forward_parent_offset: None,
            },
        );
    }

    pub fn end_parsing(mut self) -> CompletedParsing {
        self.eat_remaining_tokens();

        CompletedParsing {
            events: self.events.into_boxed_slice(),
            consumed_tokens: self.consumed_tokens,
        }
    }

    fn remap_token(&mut self, kind: TokenKind) {
        let token = self.source.token(self.consumed_tokens);

        self.consumed_tokens += 1;
        self.events.push(ParseEvent::Token {
            kind,
            length: token.lexeme_length(),
        });

        self.eat_trivia();
    }

    fn eat_trivia(&mut self) {
        loop {
            match self.current() {
                TokenKind::WhiteSpace
                | TokenKind::TombStone
                | TokenKind::CommentLine
                | TokenKind::CommentBlock => {}
                _ => break,
            }

            let token = self.source.token(self.consumed_tokens);
            self.events.push(ParseEvent::Trivia {
                kind: token.token_kind(),
                length: token.lexeme_length(),
            });
            self.consumed_tokens += 1;
        }
    }

    fn eat_remaining_tokens(&mut self) {
        if self.current() == TokenKind::EndOfFile {
            return;
        }

        if self.events.len() > 1 {
            self.events.pop(); // crack open the root element
        }

        let mut remaining = self.begin_node();

        loop {
            if self.current() == TokenKind::EndOfFile {
                break;
            }

            let token = self.source.token(self.consumed_tokens);
            self.events.push(ParseEvent::Trivia {
                kind: token.token_kind(),
                length: token.lexeme_length(),
            });
            self.consumed_tokens += 1;
        }

        self.end_node(remaining, SyntaxKind::Error("Unexpected text"));

        self.events.push(ParseEvent::EndNode); // close the root element
    }
}

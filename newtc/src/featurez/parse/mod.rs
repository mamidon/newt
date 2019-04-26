use crate::featurez::syntax::{
    BinaryExprNode, LiteralExprNode, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
    SyntaxTree, TextTreeSink, TokenSource, TreeSink,
};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

mod parse_event;
use self::parse_event::ParseEvent;

mod marker;
use self::marker::Marker;

mod parser;
pub use self::parser::Parser;

pub fn parse<F: FnOnce(&mut Parser) -> ()>(text: &str, root: F) {
    let tokens = tokenize(text);
    let source = StrTokenSource::new(tokens);
    let mut parser = Parser::new(text, source);

    root(&mut parser);
    println!("{}", parser);
}
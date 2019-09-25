use crate::featurez::runtime::VirtualMachine;
use crate::featurez::syntax::{AstNode, ExprNode, ExprVisitor, NewtValue};
use crate::featurez::syntax::{StmtNode, StmtVisitor, SyntaxElement, SyntaxTree};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

mod marker;
mod parse_event;
mod parser;
mod tests;

pub use self::marker::CompletedMarker;
pub use self::marker::Marker;
pub use self::parse_event::ParseEvent;
pub use self::parser::CompletedParsing;
pub use self::parser::Parser;

pub enum InterpretingSessionKind {
    Stmt,
    Expr,
}

pub struct InterpretingSession<'sess> {
    pub kind: InterpretingSessionKind,
    pub source: &'sess str,
}

pub fn build(session: InterpretingSession) -> SyntaxTree {
    use super::grammar::{root_expr, root_stmt};

    let tokens = tokenize(session.source);
    let source = StrTokenSource::new(tokens);
    let mut parser = Parser::new(source);

    let completed_parsing = match session.kind {
        InterpretingSessionKind::Stmt => root_stmt(parser),
        InterpretingSessionKind::Expr => root_expr(parser),
    };

    SyntaxTree::from_parser(&completed_parsing, session.source)
}

pub fn interpret(machine: &mut VirtualMachine, tree: &SyntaxTree) -> Option<NewtValue> {
    let node = match tree.root().as_node() {
        Some(n) => n,
        None => return None,
    };

    if let Some(expr) = ExprNode::cast(node) {
        return machine.visit_expr(expr).ok();
    }

    if let Some(stmt) = StmtNode::cast(node) {
        machine.visit_stmt(stmt);

        return None;
    }

    return None;
}

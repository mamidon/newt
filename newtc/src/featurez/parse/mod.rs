use crate::featurez::runtime::VirtualMachine;
use crate::featurez::runtime::RefEquality;
use crate::featurez::runtime::LexicalScopeAnalyzer;
use crate::featurez::syntax::{AstNode, ExprNode, ExprVisitor, NewtValue, NewtStaticError, NewtRuntimeError, SyntaxNode};
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
use std::collections::HashMap;


#[derive(Copy, Clone)]
pub enum InterpretingSessionKind {
    Stmt,
    Expr,
}

pub enum NewtError {
    Static(NewtStaticError),
    Runtime(NewtRuntimeError)
}

pub struct InterpretingSession<'sess> {
    kind: InterpretingSessionKind,
    source: &'sess str,
    tree: SyntaxTree,
}

impl<'sess> InterpretingSession<'sess> {
    pub fn new(kind: InterpretingSessionKind, source: &'sess str) -> InterpretingSession<'sess> {
        let tree = InterpretingSession::syntax_tree_from_source(kind, source);
        let mut resolutions_table: HashMap<RefEquality<'sess, SyntaxNode>, usize> = HashMap::new();

        InterpretingSession {
            kind,
            source,
            tree,
        }
    }

    pub fn interpret(&self, vm: &mut VirtualMachine) -> Option<NewtValue> {
        vm.interpret(&self.tree)
    }

    pub fn syntax_tree(&self) -> &SyntaxTree {
        &self.tree
    }

    fn syntax_tree_from_source(kind: InterpretingSessionKind, source: &'sess str) -> SyntaxTree {
        use super::grammar::{root_expr, root_stmt};

        let tokens = tokenize(source);
        let token_source = StrTokenSource::new(tokens);
        let mut parser = Parser::new(token_source);

        let completed_parsing = match kind {
            InterpretingSessionKind::Stmt => root_stmt(parser),
            InterpretingSessionKind::Expr => root_expr(parser),
        };

        SyntaxTree::from_parser(&completed_parsing, source)
    }
}



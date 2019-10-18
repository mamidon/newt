use crate::featurez::runtime::{VirtualMachineState, VirtualMachineInterpretingSession};
use crate::featurez::runtime::RefEquality;
use crate::featurez::runtime::LexicalScopeAnalyzer;
use crate::featurez::syntax::{AstNode, ExprNode, ExprVisitor, NewtValue, NewtStaticError, NewtRuntimeError, SyntaxNode};
use crate::featurez::syntax::{StmtNode, StmtVisitor, SyntaxElement, SyntaxTree};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};
use crate::featurez::parse::Parser;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
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
		let mut tree = InterpretingSession::syntax_tree_from_source(kind, source);
		tree.analyize();

		InterpretingSession {
			kind,
			source,
			tree,
		}
	}

	pub fn interpret(&self, vm: &mut VirtualMachineState) -> Option<NewtValue> {
		let mut session = VirtualMachineInterpretingSession::new(self.syntax_tree(), vm);
		session.interpret()
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



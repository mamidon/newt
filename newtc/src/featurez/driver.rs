use crate::featurez::runtime::{VirtualMachine, VirtualMachineInterpretingSession};
use crate::featurez::syntax::{AstNode, ExprNode, ExprVisitor, NewtValue, NewtStaticError, NewtRuntimeError, SyntaxNode, NewtResult};
use crate::featurez::syntax::{StmtNode, StmtVisitor, SyntaxElement, SyntaxTree};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};
use crate::featurez::parse::Parser;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::collections::HashMap;
use crate::featurez::grammar::{root_stmt, root_expr};


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

		InterpretingSession {
			kind,
			source,
			tree,
		}
	}

	pub fn interpret(&self, vm: &mut VirtualMachine) -> NewtResult {
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



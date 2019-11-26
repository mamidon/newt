use crate::featurez::runtime::{VirtualMachine};
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

pub enum NewtError {
	Static(NewtStaticError),
	Runtime(NewtRuntimeError)
}

use crate::featurez::parse::Parser;
use crate::featurez::runtime::VirtualMachine;
use crate::featurez::syntax::{
    AstNode, ExprNode, ExprVisitor, NewtResult, NewtRuntimeError, NewtStaticError, NewtValue,
    SyntaxNode,
};
use crate::featurez::syntax::{StmtNode, StmtVisitor, SyntaxElement, SyntaxTree};
use crate::featurez::tokens::{tokenize, StrTokenSource, Token, TokenKind};

use crate::featurez::grammar::{root_expr, root_stmt};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

pub enum NewtError {
    Static(NewtStaticError),
    Runtime(NewtRuntimeError),
}

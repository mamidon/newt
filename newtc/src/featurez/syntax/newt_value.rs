use std::ops::{Add, Sub, Mul, Div, Not, Neg};
use std::cmp::{PartialOrd, PartialEq, Ordering};
use std::str::FromStr;

use super::NewtResult;
use super::NewtRuntimeError;

use crate::featurez::TokenKind;
use crate::featurez::syntax::{
	BinaryExprNode, UnaryExprNode, LiteralExprNode, GroupingExprNode,
	AstNode, ExprNode, ExprKind,
	SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
	SyntaxTree, TextTreeSink, TokenSource, TreeSink,
};
use std::rc::Rc;


#[derive(Debug, Clone)]
pub enum NewtValue {
	Int(i64),
	Float(f64),
	Glyph(char),
	String(String),
	Bool(bool)
}

impl NewtValue {
	pub fn from_node(node: &SyntaxNode) -> Option<NewtValue> {
		ExprNode::cast(node)
			.and_then(|n| Some(n.kind()))
			.and_then(|k| if let ExprKind::LiteralExpr(l) = k { Some(l) } else { None })
			.and_then(|e| Some(NewtValue::from_literal_node(e)))
	}

	pub fn from_literal_node(node: &LiteralExprNode) -> NewtValue {
		let literal = node.literal();
		let lexeme = literal.lexeme();

		match literal.token_kind() {
			TokenKind::IntegerLiteral => NewtValue::Int(lexeme.parse().expect("unparsable literal token")),
			TokenKind::FloatLiteral => NewtValue::Float(lexeme.parse().expect("unparsable literal token")),
			TokenKind::StringLiteral => NewtValue::String(lexeme.to_string()),
			TokenKind::GlyphLiteral => NewtValue::Glyph(lexeme[1..2].parse().expect("unparsable literal token")),
			TokenKind::True => NewtValue::Bool(true),
			TokenKind::False => NewtValue::Bool(false),
			_ => panic!("Literal node has non-literal token")
		}
	}

	pub fn as_truthy(&self) -> Option<bool> {
		match self {
			NewtValue::Bool(b) => Some(*b),
			NewtValue::Int(i) => Some(*i != 0),
			_ => None
		}
	}
}

impl Add for NewtValue {
	type Output = NewtResult;

	fn add(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l + r)),
			(NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l + r)),
			_ => Err(NewtRuntimeError::TypeError)
		}
	}
}


impl Sub for NewtValue {
	type Output = NewtResult;

	fn sub(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l - r)),
			(NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l - r)),
			_ => Err(NewtRuntimeError::TypeError)
		}
	}
}


impl Mul for NewtValue {
	type Output = NewtResult;

	fn mul(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l * r)),
			(NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l - r)),
			_ => Err(NewtRuntimeError::TypeError)
		}
	}
}


impl Div for NewtValue {
	type Output = NewtResult;

	fn div(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l / r)),
			(NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l / r)),
			_ => Err(NewtRuntimeError::TypeError)
		}
	}
}


impl Not for NewtValue {
	type Output = NewtResult;

	fn not(self) -> <Self as Not>::Output {
		match self {
			_ => Err(NewtRuntimeError::TypeError)
		}
	}
}

impl Neg for NewtValue {
	type Output = NewtResult;

	fn neg(self) -> <Self as Neg>::Output {
		match self {
			NewtValue::Int(l) => Ok(NewtValue::Int(-l)),
			NewtValue::Float(l) => Ok(NewtValue::Float(-l)),
			_ => Err(NewtRuntimeError::TypeError)
		}
	}
}

impl PartialOrd for NewtValue {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(NewtValue::Int(a), NewtValue::Int(b)) => {
				if a < b {
					Some(Ordering::Less)
				} else if a == b {
					Some(Ordering::Equal)
				} else {
					Some(Ordering::Greater)
				}
			},
			(NewtValue::Float(a), NewtValue::Float(b)) => {
				if a < b {
					Some(Ordering::Less)
				} else if a == b {
					Some(Ordering::Equal)
				} else {
					Some(Ordering::Greater)
				}
			},
			_ => None
		}
	}
}

impl PartialEq for NewtValue {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(NewtValue::Int(a), NewtValue::Int(b)) => a == b,
			(NewtValue::Float(a), NewtValue::Float(b)) => a == b,
			(NewtValue::Bool(a), NewtValue::Bool(b)) => a == b,
			(NewtValue::String(a), NewtValue::String(b)) => a == b,
			(NewtValue::Glyph(a), NewtValue::Glyph(b)) => a == b,
			_ => false
		}
	}
}
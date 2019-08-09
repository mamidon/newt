use std::ops::{Add, Sub, Mul, Div, Not, Neg};
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
			_ => panic!("Literal node has non-literal token")
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

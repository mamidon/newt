use crate::featurez::TokenKind;
use crate::featurez::syntax::{
	BinaryExprNode, UnaryExprNode, LiteralExprNode, GroupingExprNode,
	AstNode, ExprNode, ExprKind,
	SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
	SyntaxTree, TextTreeSink, TokenSource, TreeSink,
};
use std::ops::{Add, Sub, Mul, Div, Not, Neg};
use std::str::FromStr;

#[derive(Debug)]
pub enum NewtValue {
	Int(i64),
	Float(f64),
	Glyph(char),
	String(String),
	RuntimeError(String)
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
			TokenKind::GlyphLiteral => NewtValue::Glyph(lexeme.parse().expect("unparsable literal token")),
			_ => panic!("Literal node has non-literal token")
		}
	}
}

// Add<Output=T>
//	+ Sub<Output=T>
//	+ Mul<Output=T>
//	+ Div<Output=T>
//	+ Not<Output=T>
//	+ Neg<Output=T>
impl Add for NewtValue {
	type Output = Self;

	fn add(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => NewtValue::Int(l + r),
			(NewtValue::Float(l), NewtValue::Float(r)) => NewtValue::Float(l + r),
			_ => NewtValue::RuntimeError(format!("Cannot add"))
		}
	}
}


impl Sub for NewtValue {
	type Output = Self;

	fn sub(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => NewtValue::Int(l - r),
			(NewtValue::Float(l), NewtValue::Float(r)) => NewtValue::Float(l - r),
			_ => NewtValue::RuntimeError(format!("Cannot sub"))
		}
	}
}


impl Mul for NewtValue {
	type Output = Self;

	fn mul(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => NewtValue::Int(l * r),
			(NewtValue::Float(l), NewtValue::Float(r)) => NewtValue::Float(l - r),
			_ => NewtValue::RuntimeError(format!("Cannot mul"))
		}
	}
}


impl Div for NewtValue {
	type Output = Self;

	fn div(self, rhs: Self) -> <Self as Add<Self>>::Output {
		match (self, rhs) {
			(NewtValue::Int(l), NewtValue::Int(r)) => NewtValue::Int(l / r),
			(NewtValue::Float(l), NewtValue::Float(r)) => NewtValue::Float(l / r),
			_ => NewtValue::RuntimeError(format!("Cannot div"))
		}
	}
}


impl Not for NewtValue {
	type Output = NewtValue;

	fn not(self) -> <Self as Not>::Output {
		match self {
			NewtValue::Int(l) => NewtValue::Int(l),
			NewtValue::Float(l) => NewtValue::Float(l),
			_ => NewtValue::RuntimeError(format!("Cannot not"))
		}
	}
}

impl Neg for NewtValue {
	type Output = NewtValue;

	fn neg(self) -> <Self as Neg>::Output {
		match self {
			NewtValue::Int(l) => NewtValue::Int(-l),
			NewtValue::Float(l) => NewtValue::Float(-l),
			_ => NewtValue::RuntimeError(format!("Cannot neg"))
		}
	}
}

pub trait ExprVisitor<T>
{
	fn visit_expr(&self, expr: &ExprNode) -> T {
		match expr.kind() {
			ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
			ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
			ExprKind::LiteralExpr(node) => self.visit_literal_expr(node),
			ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
		}
	}

	fn visit_binary_expr(&self, node: &BinaryExprNode) -> T;
	fn visit_unary_expr(&self, node: &UnaryExprNode) -> T;
	fn visit_literal_expr(&self, node: &LiteralExprNode) -> T;
	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> T;
}

pub struct ExprVirtualMachine {}

impl ExprVirtualMachine {
	pub fn new() -> ExprVirtualMachine { ExprVirtualMachine {} }
}

impl ExprVisitor<NewtValue> for ExprVirtualMachine {
	fn visit_binary_expr(&self, node: &BinaryExprNode) -> NewtValue {
		let lhs = self.visit_expr(node.lhs());
		let rhs = self.visit_expr(node.rhs());

		match node.operator() {
			TokenKind::Plus => lhs + rhs,
			TokenKind::Minus => lhs - rhs,
			TokenKind::Star => lhs * rhs,
			TokenKind::Slash => lhs / rhs,
			_ => unreachable!("not a binary")
		}
	}

	fn visit_unary_expr(&self, node: &UnaryExprNode) -> NewtValue {
		let rhs = self.visit_expr(node.rhs());

		match node.operator() {
			TokenKind::Bang => !rhs,
			TokenKind::Minus => -rhs,
			_ => unreachable!("not a unary")
		}
	}

	fn visit_literal_expr(&self, node: &LiteralExprNode) -> NewtValue {
		let literal = node.literal();
		let value = NewtValue::from_literal_node(node);

		value
	}
	
	fn visit_grouping_expr(&self, node: &GroupingExprNode) -> NewtValue {
		let expr = node.expr();
		
		self.visit_expr(expr)
	}
}

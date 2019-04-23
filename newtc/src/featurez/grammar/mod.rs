
use crate::featurez::parse::Parser;
use self::expr::*;

mod expr {
	use crate::featurez::parse::Parser;
	use crate::featurez::syntax::{
		SyntaxElement, 
		SyntaxToken, 
		SyntaxNode, 
		SyntaxKind
	};
	use crate::featurez::{Token, TokenKind};

	pub fn expr(p: &mut Parser) {
		let mut start = p.begin_node();
		
		add_expr(p);
		
		p.end_node(&mut start, SyntaxKind::BinaryExpr);
	}

	pub fn add_expr(p: &mut Parser) -> Option<SyntaxKind> {
		let mut start = p.begin_node();

		let left = integer_literal_expr(p);
		
		match p.current().token_kind() {
			TokenKind::Plus
			| TokenKind::Minus => {
				p.token(p.current());
				
				let right = integer_literal_expr(p);
				p.end_node(&mut start, SyntaxKind::BinaryExpr);
			}, 
			_ => start.abandon()
		}
		
		return None;
	}
	
	pub fn mult_expr(p: &mut Parser) -> Option<SyntaxKind> {
		let mut start = p.begin_node();
		
		let left = unary_expr(p);
	}
	
	pub fn unary_expr(p: &mut Parser) -> Option<SyntaxKind> {
		if let Some(primary_token) = primary_expr(p) {
			return Some(primary_token);
		}
		
		match p.current().token_kind() {
			TokenKind::Bang
			| TokenKind::Minus => {
				let mut start = p.begin_node();
				let expr = expr(p);
				p.end_node(&mut start, SyntaxKind::UnaryExpr);
				
				return Some(SyntaxKind::UnaryExpr);
			},
			_ => return None;
		}
	}
	
	pub fn primary_expr(p: &mut Parser) -> Option<SyntaxKind> {
		if let integer = integer_literal_expr(p) {
			return integer;
		} else {
			return None;
		}
	}

	pub fn integer_literal_expr(p: &mut Parser) -> Option<SyntaxKind> {
		if p.current().token_kind() == TokenKind::IntegerLiteral {
			let mut start = p.begin_node();
			p.token(p.current());
			p.end_node(&mut start, SyntaxKind::LiteralExpr);
			
			return Some(SyntaxKind::LiteralExpr);
		}
		
		return None;
	}
}

pub fn root(p: &mut Parser) {
	expr(p);
}

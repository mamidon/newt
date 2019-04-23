
use crate::featurez::parse::Parser;
use self::expr::*;

mod expr {
	use crate::featurez::parse::Parser;
	use crate::featurez::syntax::{SyntaxNode, SyntaxKind};
	use crate::featurez::{Token, TokenKind};

	pub fn expr(p: &mut Parser) {
		let mut start = p.begin_node();
		add_expr(p);
		p.end_node(&mut start, SyntaxKind::BinaryExpr);
	}

	pub fn add_expr(p: &mut Parser) {

		let mut start = p.begin_node();

		let left = integer_literal_expr(p);
		let operator = plus_op(p);
		let right = integer_literal_expr(p);

		p.end_node(&mut start, SyntaxKind::PlusExpr);
	}

	pub fn integer_literal_expr(p: &mut Parser) {
		if p.current().token_kind() == TokenKind::IntegerLiteral {
			let mut start = p.begin_node();
			p.token(p.current());
			p.end_node(&mut start, SyntaxKind::LiteralExpr);
		}
	}

	pub fn plus_op(p: &mut Parser) {
		if p.current().token_kind() == TokenKind::Plus {
			p.token(p.current());
		}
	}
}

pub fn root(p: &mut Parser) {
	expr(p);
}

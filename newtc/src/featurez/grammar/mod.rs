use self::expr::*;
use crate::featurez::parse::Parser;

mod expr {
    use crate::featurez::parse::Parser;
    use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
    use crate::featurez::{Token, TokenKind};

    pub fn expr(p: &mut Parser) {
        let mut start = p.begin_node();

        add_expr(p);

        p.end_node(&mut start, SyntaxKind::Expr);
    }

    pub fn add_expr(p: &mut Parser) {
		let mut start = p.begin_node();

		mult_expr(p);

		if p.token_if(TokenKind::Plus) || p.token_if(TokenKind::Minus) {
			expr(p);
			p.end_node(&mut start, SyntaxKind::BinaryExpr);
		} else {
			start.abandon();
		}
    }

    pub fn mult_expr(p: &mut Parser) {
		let mut start = p.begin_node();

		unary_expr(p);

		if p.token_if(TokenKind::Star) || p.token_if(TokenKind::Slash) {
			expr(p);
			p.end_node(&mut start, SyntaxKind::BinaryExpr);
		} else {
			start.abandon();
		}
    }

    pub fn unary_expr(p: &mut Parser) {
		let mut start = p.begin_node();
		
		if p.token_if(TokenKind::Bang) || p.token_if(TokenKind::Minus) {
			let expr = expr(p);
			
			p.end_node(&mut start, SyntaxKind::UnaryExpr);
		} else {
			start.abandon();
			
			primary_expr(p);
		}
    }

    pub fn primary_expr(p: &mut Parser) -> Option<SyntaxKind> {
        if let Some(integer) = integer_literal_expr(p) {
            return Some(integer);
        } else {
            return None;
        }
    }

    pub fn integer_literal_expr(p: &mut Parser) -> Option<SyntaxKind> {
		let mut start = p.begin_node();
		
		if p.token_if(TokenKind::IntegerLiteral) {
			
			p.end_node(&mut start, SyntaxKind::LiteralExpr);
			
			return Some(SyntaxKind::LiteralExpr);
		}
		
        return None;
    }
}

pub fn root(p: &mut Parser) {
    expr(p);
}

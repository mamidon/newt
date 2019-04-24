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

        match p.current().token_kind() {
            TokenKind::Plus | TokenKind::Minus => {
                p.token(p.current());

                expr(p);
                p.end_node(&mut start, SyntaxKind::BinaryExpr);
            }
            _ => start.abandon(),
        }
    }

    pub fn mult_expr(p: &mut Parser) {
		let mut start = p.begin_node();

		unary_expr(p);

		match p.current().token_kind() {
			TokenKind::Star | TokenKind::Slash => {
				p.token(p.current());

				expr(p);
				p.end_node(&mut start, SyntaxKind::BinaryExpr);
			}
			_ => start.abandon(),
		}
    }

    pub fn unary_expr(p: &mut Parser) {
		match p.current().token_kind() {
			TokenKind::Bang
			| TokenKind::Minus => {
				let mut start = p.begin_node();
				
				p.token(p.current());
				let expr = expr(p);
				
				p.end_node(&mut start, SyntaxKind::UnaryExpr);
			},
			_ => { primary_expr(p); }
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

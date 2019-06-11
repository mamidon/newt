use self::expr::*;
use crate::featurez::parse::Parser;

mod expr {
    use crate::featurez::parse::Parser;
	use crate::featurez::parse::CompletedMarker;
    use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
    use crate::featurez::{Token, TokenKind};

	const PRECEDENCE_TABLE: &[(&[TokenKind], fn(&mut Parser, Option<CompletedMarker>) -> CompletedMarker)] = &[
		(&[TokenKind::IntegerLiteral], primary_expr2),
		(&[TokenKind::Minus, TokenKind::Bang], unary_expr2),
		(&[TokenKind::Star, TokenKind::Slash], mult_expr2),
		(&[TokenKind::Plus, TokenKind::Minus], add_expr2),
	];
	
    pub fn expr(p: &mut Parser) {
		expr_core(p, PRECEDENCE_TABLE.len() - 1);
    }
	
	// https://en.wikipedia.org/wiki/Operator-precedence_parser#Example_execution_of_the_algorithm
	// I've got the sort of right idea here -- but my notion of precedence is backwards 
	// additionally I only need an explicit function for parsing primary expressions, but I 
	// do need to encode the precedence & associativity of operators
	// also see http://craftinginterpreters.com/compiling-expressions.html#a-pratt-parser
	fn expr_core(p: &mut Parser, precedence: usize) -> Option<CompletedMarker> {
		if precedence >= PRECEDENCE_TABLE.len() {
			return None;
		}
		
		let (acceptable_tokens, handler) 
			= PRECEDENCE_TABLE[precedence];
		
		let mut preceding_expr: Option<CompletedMarker> = None;
	
		if !acceptable_tokens.contains(&p.current()) {
			preceding_expr = expr_core(p, precedence - 1);
		}
		
		loop {	
			if acceptable_tokens.contains(&p.current()) {
				return Some(handler(p, preceding_expr));
			} else {
				return preceding_expr;
			}
		}
	}
	
	fn add_expr2(p: &mut Parser, preceding_expr: Option<CompletedMarker>) -> CompletedMarker {
		let mut node = p.begin_node();
		
		p.expect_token_kind_in(&[TokenKind::Plus, TokenKind::Minus], "Shouldn't happen");
		p.precede_node(&mut preceding_expr.unwrap(), &node);
		
		expr_core(p, 3);
		
		p.end_node(node, SyntaxKind::BinaryExpr)
	}

	fn mult_expr2(p: &mut Parser, preceding_expr: Option<CompletedMarker>) -> CompletedMarker {
		let mut node = p.begin_node();

		p.expect_token_kind_in(&[TokenKind::Star, TokenKind::Slash], "Shouldn't happen");
		p.precede_node(&mut preceding_expr.unwrap(), &node);
		
		expr_core(p, 2);
		
		p.end_node(node, SyntaxKind::BinaryExpr)
	}

	fn unary_expr2(p: &mut Parser, preceding_expr: Option<CompletedMarker>) -> CompletedMarker {
		let mut node = p.begin_node();

		p.expect_token_kind_in(&[TokenKind::Bang, TokenKind::Minus], "Shouldn't happen");
		p.precede_node(&mut preceding_expr.unwrap(), &node);

		expr_core(p, 1);
		
		p.end_node(node, SyntaxKind::UnaryExpr)
	}

	fn primary_expr2(p: &mut Parser, preceding_expr: Option<CompletedMarker>) -> CompletedMarker {
		let mut node = p.begin_node();

		p.expect_token_kind_in(&[TokenKind::IntegerLiteral], "Shouldn't happen");

		p.end_node(node, SyntaxKind::LiteralExpr)
	}

    pub fn add_expr(p: &mut Parser) {
		let mut start = p.begin_node();

		mult_expr(p);
		if p.token_if(TokenKind::Plus) || p.token_if(TokenKind::Minus) {
			add_expr(p);
			p.end_node(start, SyntaxKind::BinaryExpr);
		} else {
			start.abandon();
		}
    }

    pub fn mult_expr(p: &mut Parser) {
		let mut start = p.begin_node();

		unary_expr(p);
		
		if p.token_if(TokenKind::Star) || p.token_if(TokenKind::Slash) {
			while p.token_if(TokenKind::Star) || p.token_if(TokenKind::Slash) {
				let mut inner = p.begin_node();
				unary_expr(p);
				p.end_node(inner, SyntaxKind::BinaryExpr);
			}
			
			p.end_node(start, SyntaxKind::BinaryExpr);
		} else {
			start.abandon();
		}
    }

    pub fn unary_expr(p: &mut Parser) {
		let mut start = p.begin_node();
		
		if p.token_if(TokenKind::Bang) || p.token_if(TokenKind::Minus) {
			let expr = expr(p);
			
			p.end_node(start, SyntaxKind::UnaryExpr);
		} else {
			start.abandon();
			
			primary_expr(p);
		}
    }

    pub fn primary_expr(p: &mut Parser) {
        integer_literal_expr(p);
    }

    pub fn integer_literal_expr(p: &mut Parser) {
		let mut start = p.begin_node();
		
		p.expect_token_kind(TokenKind::IntegerLiteral, "Expected integer literal");
		
		p.end_node(start, SyntaxKind::LiteralExpr);
    }
}

pub fn root(p: &mut Parser) {
    expr(p);
}

use self::expr::*;
use crate::featurez::parse::Parser;

mod expr {
    use crate::featurez::parse::Parser;
	use crate::featurez::parse::CompletedMarker;
    use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
    use crate::featurez::{Token, TokenKind};
	use std::collections::HashMap;

	type BinaryOperatorHandler =  fn(&mut Parser, CompletedMarker) -> CompletedMarker;
	type UnaryOperatorHandler = fn(&mut Parser) -> CompletedMarker;
	type PrecedenceTable = HashMap<TokenKind, (usize, bool, bool)>;
	
	lazy_static! {
		static ref PRECEDENCE_TABLE: PrecedenceTable = [
			(TokenKind::IntegerLiteral, 0, false, true),
			(TokenKind::Star, 2, true, false),
			(TokenKind::Slash, 2, true, false),
			(TokenKind::Plus, 3, true, false),
			(TokenKind::Minus, 3, true, true),
			(TokenKind::Bang, 1, false, true),
		].iter()
			.map(|tuple| (tuple.0, (tuple.1, tuple.2, tuple.3)))
			.collect::<HashMap<_,_>>();
	}
	
    pub fn expr(p: &mut Parser) {
		let lhs = primary_expr(p);
		
		expr_core(p, lhs, 3);
    }
	
	// https://en.wikipedia.org/wiki/Operator-precedence_parser#Example_execution_of_the_algorithm
	// also see http://craftinginterpreters.com/compiling-expressions.html#a-pratt-parser
	fn expr_core(p: &mut Parser, first_lhs: CompletedMarker, precedence: usize) -> CompletedMarker {
		let mut lookahead = p.current();
		let mut lhs = first_lhs;
		
		while lookahead.is_binary_operator() && lookahead_precedence(lookahead) <= precedence {
			let mut node = p.begin_node();
			p.precede_node(&mut lhs, &node);
			p.token_if(lookahead);
			let rhs = primary_expr(p);

			lookahead = p.current();

			if lookahead.is_binary_operator() && lookahead_precedence(lookahead) < precedence {
				expr_core(p, rhs, lookahead_precedence(lookahead));
				
				lookahead = p.current();
			}
			
			lhs = p.end_node(node, SyntaxKind::BinaryExpr);
		}
		
		lhs
	}
	
	fn lookahead_precedence(token: TokenKind) -> usize {
		PRECEDENCE_TABLE[&token].0
	}
	
	fn lookahead_binary(token: TokenKind) -> bool {
		PRECEDENCE_TABLE[&token].1
	}

	fn lookahead_unary(token: TokenKind) -> bool {
		PRECEDENCE_TABLE[&token].2
	}

	fn primary_expr(p: &mut Parser) -> CompletedMarker {
		let mut node = p.begin_node();
		
		let completed = match p.current() {
			TokenKind::Bang => {
				p.token_if(TokenKind::Bang);
				expr(p);
				
				p.end_node(node, SyntaxKind::UnaryExpr)
			},
			TokenKind::Minus => {
				p.token_if(TokenKind::Minus);
				expr(p);
				
				p.end_node(node, SyntaxKind::UnaryExpr)
			}, 
			TokenKind::IntegerLiteral => {
				p.token_if(TokenKind::IntegerLiteral);
				
				p.end_node(node, SyntaxKind::LiteralExpr)
			},
			TokenKind::FloatLiteral => {
				p.token_if(TokenKind::FloatLiteral);

				p.end_node(node, SyntaxKind::LiteralExpr)
			},
			TokenKind::GlyphLiteral => {
				p.token_if(TokenKind::GlyphLiteral);
				
				p.end_node(node, SyntaxKind::LiteralExpr)
			},
			TokenKind::StringLiteral => {
				p.token_if(TokenKind::StringLiteral);
				
				p.end_node(node, SyntaxKind::LiteralExpr)
			},
			TokenKind::LeftParenthesis => {
				p.token_if(TokenKind::LeftParenthesis);
				expr(p);
				p.expect_token_kind(TokenKind::RightParenthesis, "Expected ')' to close grouping");
				
				p.end_node(node, SyntaxKind::GroupingExpr)
			},
			// TODO identifiers, function calls
			_ => {
				p.expect_token_kind_in(&[], "Expected a primary expression");
				
				p.end_node(node, SyntaxKind::LiteralExpr)
			}
		};

		completed		
	}
}

pub fn root(p: &mut Parser) {
    expr(p);
}

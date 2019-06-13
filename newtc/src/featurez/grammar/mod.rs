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
			(TokenKind::Plus, 3, true, false),
			(TokenKind::Minus, 3, true, true),
			(TokenKind::Bang, 1, false, true),
		].iter()
			.map(|tuple| (tuple.0, (tuple.1, tuple.2, tuple.3)))
			.collect::<HashMap<_,_>>();
	}
	
    pub fn expr(p: &mut Parser) {
		let lhs = primary_expr(p);
		
		expr_core(p, lhs);
    }
	
	// https://en.wikipedia.org/wiki/Operator-precedence_parser#Example_execution_of_the_algorithm
	// I've got the sort of right idea here -- but my notion of precedence is backwards 
	// additionally I only need an explicit function for parsing primary expressions, but I 
	// do need to encode the precedence & associativity of operators
	// also see http://craftinginterpreters.com/compiling-expressions.html#a-pratt-parser
	fn expr_core(p: &mut Parser, first_lhs: CompletedMarker) -> CompletedMarker {
		let mut lookahead = p.current();
		let mut lhs = first_lhs;
		
		while lookahead == TokenKind::Plus {
			let mut node = p.begin_node();
			p.precede_node(&mut lhs, &node);
			p.token_if(lookahead);
			let rhs = primary_expr(p);
			
			lhs = p.end_node(node, SyntaxKind::BinaryExpr);
			
			lookahead = p.current();
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

		p.expect_token_kind_in(&[TokenKind::IntegerLiteral], "Shouldn't happen");

		p.end_node(node, SyntaxKind::LiteralExpr)
	}
}

pub fn root(p: &mut Parser) {
    expr(p);
}

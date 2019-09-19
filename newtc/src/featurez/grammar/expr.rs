use crate::featurez::parse::Parser;
use crate::featurez::parse::CompletedMarker;
use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTree};
use crate::featurez::{Token, TokenKind};
use std::collections::HashMap;

type PrecedenceTable = HashMap<TokenKind, (usize, bool)>;

lazy_static! {
	static ref PRECEDENCE_TABLE: PrecedenceTable = [
		(TokenKind::Star, 2, true),
		(TokenKind::Slash, 2, true),
		(TokenKind::Plus, 3, true),
		(TokenKind::Minus, 3, true),
		//TODO REGRESSION -- this breaks left associativity somehow
		/*(TokenKind::Less, 4, true),
		(TokenKind::LessEquals, 4, true),
		(TokenKind::Greater, 4, true),
		(TokenKind::GreaterEquals, 4, true),
		(TokenKind::EqualsEquals, 4, true),*/
		(TokenKind::Bang, 1, false),
	].iter()
		.map(|tuple| (tuple.0, (tuple.1, tuple.2)))
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
		TokenKind::True => {
			p.token_if(TokenKind::True);
			p.end_node(node, SyntaxKind::LiteralExpr)
		},
		TokenKind::False => {
			p.token_if(TokenKind::False);
			p.end_node(node, SyntaxKind::LiteralExpr)
		},
		TokenKind::LeftParenthesis => {
			p.token_if(TokenKind::LeftParenthesis);
			expr(p);
			p.expect_token_kind(TokenKind::RightParenthesis, "Expected ')' to close grouping");
			
			p.end_node(node, SyntaxKind::GroupingExpr)
		},
		TokenKind::Identifier => {
			p.token_if(TokenKind::Identifier);

			p.end_node(node, SyntaxKind::VariableExpr)
		},
		// TODO function calls
		_ => {
			p.expect_token_kind_in(&[], "Expected a primary expression");
			
			p.end_node(node, SyntaxKind::LiteralExpr)
		}
	};

	completed		
}

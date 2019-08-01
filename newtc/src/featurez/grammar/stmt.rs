use crate::featurez::parse::{Parser, Marker};
use crate::featurez::parse::CompletedMarker;
use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
use crate::featurez::{Token, TokenKind};

use super::expr;

pub fn stmt(p: &mut Parser) {
	let starting_stmt_tokens = &[
		TokenKind::Fn,
		TokenKind::For,
		TokenKind::If,
		TokenKind::Let,
		TokenKind::Return,
		TokenKind::While,
	];
	
	let node = p.begin_node();
	
	if p.token_if(TokenKind::Let) {
		stmt_let(p, node);	
	}
	
}

fn stmt_let(p: &mut Parser, node: Marker) {
	p.expect_token_kind(TokenKind::Identifier, "Expected identifier");
	p.expect_token_kind(TokenKind::Equals, "Expected equals");
	
	expr(p);
	
	p.expect_token_kind(TokenKind::SemiColon, "Expected semi-colon");
	p.end_node(node, SyntaxKind::VariableDeclarationStmt);
}

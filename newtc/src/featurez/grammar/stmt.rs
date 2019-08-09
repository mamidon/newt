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
		TokenKind::LeftBrace
	];
	
	let node = p.begin_node();

	match p.current() {
		TokenKind::Let => stmt_let(p, node),
		TokenKind::LeftBrace => stmt_list(p, node),
		_ => stmt_expr(p, node)
	}
}

fn stmt_expr(p: &mut Parser, node: Marker) {
	expr(p);

	p.expect_token_kind(TokenKind::SemiColon, "Expected ';'");
	p.end_node(node, SyntaxKind::BinaryExpr);
}

fn stmt_list(p: &mut Parser, node: Marker) {
	p.expect_token_kind(TokenKind::LeftBrace, "Expected '{'");

	while !p.token_if(TokenKind::RightBrace) {
		stmt(p);
	}

	p.end_node(node, SyntaxKind::StmtListStmt);
}

fn stmt_let(p: &mut Parser, node: Marker) {
	p.expect_token_kind(TokenKind::Let, "Expected 'let'");
	p.expect_token_kind(TokenKind::Identifier, "Expected identifier");
	p.expect_token_kind(TokenKind::Equals, "Expected equals");
	
	expr(p);
	
	p.expect_token_kind(TokenKind::SemiColon, "Expected semi-colon");
	p.end_node(node, SyntaxKind::VariableDeclarationStmt);
}

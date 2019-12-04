use crate::featurez::parse::CompletedMarker;
use crate::featurez::parse::{CompletedParsing, Marker, Parser};
use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, RValNode};
use crate::featurez::{Token, TokenKind};

use super::expr;
use crate::featurez::grammar::root_expr;
use crate::featurez::grammar::expr::primary_expr;

pub fn stmt(p: &mut Parser) {
    let starting_stmt_tokens = &[
        TokenKind::Fn,
        TokenKind::For,
        TokenKind::If,
        TokenKind::Let,
        TokenKind::Return,
        TokenKind::While,
        TokenKind::LeftBrace,
    ];

    let node = p.begin_node();

    match p.current() {
        TokenKind::Let => stmt_let(p, node),
        TokenKind::LeftBrace => stmt_list(p, node),
        TokenKind::If => stmt_if(p, node),
        TokenKind::While => stmt_while(p, node),
        TokenKind::Fn => stmt_fn(p, node),
        TokenKind::Return => stmt_return(p, node),
        _ => variable_stmt(p, node),
    }
}

fn stmt_return(p: &mut Parser, node: Marker) {
    p.token(TokenKind::Return);

    if p.current() != TokenKind::SemiColon {
        expr(p);
    }

    p.token(TokenKind::SemiColon);
    p.end_node(node, SyntaxKind::ReturnStmt);
}

fn stmt_fn(p: &mut Parser, node: Marker) {
    p.token(TokenKind::Fn);
    p.token(TokenKind::Identifier);
    p.token(TokenKind::LeftParenthesis);

    if !p.token_if(TokenKind::RightParenthesis) {
        p.token(TokenKind::Identifier);

        while !p.token_if(TokenKind::RightParenthesis) {
            p.token(TokenKind::Comma);
            p.token(TokenKind::Identifier);
        }
    }

    let mut stmt_list_node = p.begin_node();
    stmt_list(p, stmt_list_node);
    p.end_node(node, SyntaxKind::FunctionDeclarationStmt);
}

fn stmt_while(p: &mut Parser, node: Marker) {
    p.token(TokenKind::While);

    expr(p);

    let stmts = p.begin_node();
    stmt_list(p, stmts);

    p.end_node(node, SyntaxKind::WhileStmt);
}

fn stmt_if(p: &mut Parser, node: Marker) {
    p.token(TokenKind::If);

    expr(p);

    let truth_list = p.begin_node();
    stmt_list(p, truth_list);

    if p.token_if(TokenKind::Else) {
        let false_list = p.begin_node();
        stmt_list(p, false_list);
    }

    p.end_node(node, SyntaxKind::IfStmt);
}

fn variable_stmt(p: &mut Parser, node: Marker) {
    if p.current2() == Some((TokenKind::Identifier, TokenKind::Equals)) {
        stmt_assignment(p, node);
    } else {
        stmt_expr(p, node);
    }
}

fn stmt_assignment(p: &mut Parser, node: Marker) {
    rval(p);

    p.token(TokenKind::Equals);

    expr(p);

    p.expect_token_kind(TokenKind::SemiColon, "Expected ';'");
    p.end_node(node, SyntaxKind::AssignmentStmt);
}

fn stmt_expr(p: &mut Parser, node: Marker) {
    expr(p);

    p.expect_token_kind(TokenKind::SemiColon, "Expected ';'");
    p.end_node(node, SyntaxKind::ExprStmt);
}

fn stmt_list(p: &mut Parser, node: Marker) {
    p.expect_token_kind(TokenKind::LeftBrace, "Expected '{'");

    loop {
        if p.token_if(TokenKind::RightBrace) {
            break;
        }

        if p.current() == TokenKind::EndOfFile {
            p.expect_token_kind(TokenKind::RightBrace, "Expected '}'");
            break;
        }

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

fn rval(p: &mut Parser) {
    let marker = primary_expr(p);

    let remap_target = match marker.kind() {
        SyntaxKind::ObjectPropertyExpr => SyntaxKind::ObjectPropertyRVal,
        SyntaxKind::VariableExpr => SyntaxKind::VariableRval,
        _ => SyntaxKind::Error("Expected an acceptable r-val")
    };

    p.remap_node(&marker, remap_target);
}

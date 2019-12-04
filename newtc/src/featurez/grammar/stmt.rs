use crate::featurez::parse::CompletedMarker;
use crate::featurez::parse::{CompletedParsing, Marker, Parser};
use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, RValNode, ExprNode};
use crate::featurez::{Token, TokenKind};

use super::expr;
use crate::featurez::grammar::root_expr;

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
        _ => stmt_assignment_or_expr(p, node),
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

/**
This function needs to parse a node which could be either an
    assignment to a variable 'x = 42;'
    an assignment to a property 'x.y = 42;'
    an expression statement 'x.y();'

    We start by parsing an expression.
    If it's a variable or property expression and the next token is an equals then
        we parse an assignment statement
    If it's any expression and followed by an equals token then
        it's a error -- assignment to a non-rval
    If it's any expression and followed by a semi-colon then
        it's an expression statement
    Otherwise it's an error
*/
fn stmt_assignment_or_expr(p: &mut Parser, node: Marker) {
    let expr = expr(p);

    match p.current() {
        TokenKind::Equals => stmt_assignment(p, node, expr),
        TokenKind::SemiColon => stmt_expr(p, node),
        _ => {
            p.expect_token_kind(TokenKind::SemiColon, "Expected ';'");
            p.end_node(node, SyntaxKind::Error("Expected a valid statement"));
        }
    }
}

fn stmt_assignment(p: &mut Parser, node: Marker, rval_expr: CompletedMarker) {
    rval(p, rval_expr);

    p.token(TokenKind::Equals);

    expr(p);

    p.expect_token_kind(TokenKind::SemiColon, "Expected ';'");
    p.end_node(node, SyntaxKind::AssignmentStmt);
}

fn stmt_expr(p: &mut Parser, node: Marker) {
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

fn rval(p: &mut Parser, rval: CompletedMarker) {
    let remap_target = match rval.kind() {
        SyntaxKind::ObjectPropertyExpr => SyntaxKind::ObjectPropertyRVal,
        SyntaxKind::VariableExpr => SyntaxKind::VariableRval,
        _ => SyntaxKind::Error("Expected an acceptable r-val")
    };

    p.remap_node(&rval, remap_target);
}

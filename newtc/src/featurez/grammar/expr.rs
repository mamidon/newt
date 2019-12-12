use crate::featurez::parse::{CompletedMarker, Marker};
use crate::featurez::parse::Parser;
use crate::featurez::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTree};
use crate::featurez::{Token, TokenKind};
use std::collections::HashMap;
use crate::featurez::tokens::TokenKind::TombStone;

type RhsParseFunction = fn(&mut Parser, CompletedMarker) -> CompletedMarker;
type LhsParseFunction = fn(&mut Parser) -> CompletedMarker;
type PrecedenceLevel = usize;

struct PrecedenceRule {
    token_kind: TokenKind,
    precedence: PrecedenceLevel,
    unary_prefix: Option<LhsParseFunction>,
    binary: Option<RhsParseFunction>,
    unary_suffix: Option<RhsParseFunction>
}

impl PrecedenceRule {
    pub fn rule(precedence: PrecedenceLevel, token_kind:
                TokenKind,
                unary_prefix: Option<LhsParseFunction>,
                binary: Option<RhsParseFunction>,
                unary_suffix: Option<RhsParseFunction>) -> PrecedenceRule {

        PrecedenceRule {
            precedence,
            token_kind,
            unary_prefix,
            binary,
            unary_suffix
        }
    }
}

lazy_static! {
    static ref PRECEDENCE_RULES: HashMap<TokenKind, PrecedenceRule> = build_precedence_rules_table();
}

fn get_prefix(token_kind: TokenKind) -> Option<LhsParseFunction> {
    PRECEDENCE_RULES.get(&token_kind).and_then(|rule| rule.unary_prefix)
}

fn get_binary(token_kind: TokenKind) -> Option<RhsParseFunction> {
    PRECEDENCE_RULES.get(&token_kind).and_then(|rule| rule.binary)
}

fn get_suffix(token_kind: TokenKind) -> Option<RhsParseFunction> {
    PRECEDENCE_RULES.get(&token_kind).and_then(|rule| rule.unary_suffix)
}

fn get_precedence(token_kind: TokenKind) -> PrecedenceLevel {
    PRECEDENCE_RULES.get(&token_kind).map(|rule| rule.precedence).unwrap_or(NO_PRECEDENCE)
}

const PRIMARY_PRECEDENCE: PrecedenceLevel = 1;
const MULTIPLICATION_PRECEDENCE: PrecedenceLevel = 2;
const ADDITION_PRECEDENCE: PrecedenceLevel = 3;
const LOGIC_PRECEDENCE: PrecedenceLevel = 4;
const NO_PRECEDENCE: PrecedenceLevel = 1000;

fn build_precedence_rules_table() -> HashMap<TokenKind, PrecedenceRule> {
    vec![
        // basic math operators
        PrecedenceRule::rule(
            MULTIPLICATION_PRECEDENCE,
            TokenKind::Star,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            MULTIPLICATION_PRECEDENCE,
            TokenKind::Slash,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            ADDITION_PRECEDENCE,
            TokenKind::Plus,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            ADDITION_PRECEDENCE,
            TokenKind::Minus,
            Some(unary_prefix), Some(binary_expr), None),

        // logical operators
        PrecedenceRule::rule(
            LOGIC_PRECEDENCE,
            TokenKind::LessEquals,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            LOGIC_PRECEDENCE,
            TokenKind::Less,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            LOGIC_PRECEDENCE,
            TokenKind::GreaterEquals,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            LOGIC_PRECEDENCE,
            TokenKind::Greater,
            None, Some(binary_expr), None),

        PrecedenceRule::rule(
            LOGIC_PRECEDENCE,
            TokenKind::EqualsEquals,
            None, Some(binary_expr), None),

        // primaries
        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::Bang,
            Some(unary_prefix), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::IntegerLiteral,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::FloatLiteral,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::GlyphLiteral,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::StringLiteral,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::Identifier,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::True,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            PRIMARY_PRECEDENCE,
            TokenKind::False,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            NO_PRECEDENCE,
            TokenKind::LeftParenthesis,
            Some(primary_expr), None, Some(unary_suffix)),

        PrecedenceRule::rule(
            NO_PRECEDENCE,
            TokenKind::LeftBrace,
            Some(primary_expr), None, None),

        PrecedenceRule::rule(
            NO_PRECEDENCE,
            TokenKind::Dot,
            None, None, Some(unary_suffix))

    ].into_iter()
        .map(|rule| (rule.token_kind, rule))
        .collect()
}

pub fn expr(p: &mut Parser) -> CompletedMarker {
    parse_expr(p, LOGIC_PRECEDENCE)
}

fn parse_expr(p: &mut Parser, precedence: PrecedenceLevel) -> CompletedMarker {
    let prefix_parser = if let Some(prefix_parser) = get_prefix(p.current()) {
        prefix_parser
    } else {
        let marker = p.begin_node();
        return p.end_node(marker, SyntaxKind::Error("Expected expression"));
    };

    let mut lhs = prefix_parser(p);

    while precedence >= get_precedence(p.current()) {
        let binary_parser = get_binary(p.current())
            .expect(format!("Precedence has no meaning without binary operators for {:?}", p.current()).as_str());

        lhs = binary_parser(p, lhs);
    }

    lhs
}

fn binary_expr(p: &mut Parser, mut lhs: CompletedMarker) -> CompletedMarker {
    let marker = p.begin_node();
    p.precede_node(&mut lhs, &marker);
    let operator = p.current();
    p.token(operator);
    parse_expr(p, get_precedence(operator) - 1);

    p.end_node(marker, SyntaxKind::BinaryExpr)
}

fn unary_prefix(p: &mut Parser) -> CompletedMarker {
    let marker = p.begin_node();
    let operator = p.current();
    p.token(operator);
    parse_expr(p, get_precedence(operator) - 1);

    p.end_node(marker, SyntaxKind::UnaryExpr)
}

fn unary_suffix(p: &mut Parser, mut marker: CompletedMarker) -> CompletedMarker {
    match p.current() {
	    TokenKind::Dot => object_property_expr(p, marker),
	    TokenKind::LeftParenthesis => call_expr(p, marker),
	    _ => {
		    let error = p.begin_node();
		    p.end_node(error, SyntaxKind::Error("Expected unary suffix expression, got {:?}"))
	    }
    }
}

fn literal_expr(p: &mut Parser) -> CompletedMarker {
    let mut node = p.begin_node();
    p.token(p.current());
    p.end_node(node, SyntaxKind::PrimitiveLiteralExpr)
}

fn grouping_expr(p: &mut Parser) -> CompletedMarker {
    let mut node = p.begin_node();
    p.token_if(TokenKind::LeftParenthesis);
    expr(p);
    p.expect_token_kind(
        TokenKind::RightParenthesis,
        "Expected ')' to close grouping",
    );

    p.end_node(node, SyntaxKind::GroupingExpr)
}

fn variable_expr(p: &mut Parser) -> CompletedMarker {
    let mut node = p.begin_node();
    p.token(p.current());
    p.end_node(node, SyntaxKind::VariableExpr)
}

fn primary_expr(p: &mut Parser) -> CompletedMarker {
    let mut completed = match p.current() {
        TokenKind::IntegerLiteral => {
            literal_expr(p)
        }
        TokenKind::FloatLiteral => {
            literal_expr(p)
        }
        TokenKind::GlyphLiteral => {
            literal_expr(p)
        }
        TokenKind::StringLiteral => {
            literal_expr(p)
        }
        TokenKind::True => {
	        literal_expr(p)
        }
        TokenKind::False => {
	        literal_expr(p)
        }
        TokenKind::LeftParenthesis => {
            grouping_expr(p)
        }
        TokenKind::Identifier => {
            variable_expr(p)
        }
        TokenKind::LeftBrace => {
            object_literal_expr(p)
        }
        _ => {
	        let mut error = p.begin_node();
            p.end_node(error, SyntaxKind::Error("Expected primary expression"))
        }
    };

    while let Some(unary_suffix) = get_suffix(p.current()) {
        completed = unary_suffix(p, completed);
    }

    completed
}

fn object_property_expr(p: &mut Parser, mut node: CompletedMarker) -> CompletedMarker {
	let mut previous = node;
	while p.current() == TokenKind::Dot {
		let next = p.begin_node();
		p.precede_node(&mut previous, &next);

		p.token(TokenKind::Dot);
		p.token(TokenKind::Identifier);

		previous = p.end_node(next, SyntaxKind::ObjectPropertyExpr);
	}

	previous
}

fn object_literal_expr(p: &mut Parser) -> CompletedMarker {
   let node = p.begin_node();
    p.token(TokenKind::LeftBrace);

    if p.current() == TokenKind::Identifier {
        p.token(TokenKind::Identifier);
        p.token(TokenKind::Colon);
        expr(p);
    }

    while p.current() == TokenKind::Comma && p.current() != TokenKind::EndOfFile {
        p.token(TokenKind::Comma);
        p.token(TokenKind::Identifier);
        p.token(TokenKind::Colon);
        expr(p);
    }

    p.token(TokenKind::RightBrace);

    p.end_node(node, SyntaxKind::ObjectLiteralExpr)
}

fn call_expr(p: &mut Parser, mut lhs: CompletedMarker) -> CompletedMarker {
    let mut call_begin = p.begin_node();
    p.precede_node(&mut lhs, &call_begin);

    p.expect_token_kind(TokenKind::LeftParenthesis, "Expected '('");
    if p.current() != TokenKind::EndOfFile && p.current() != TokenKind::RightParenthesis {
        expr(p);
    }

    while p.current() != TokenKind::EndOfFile && p.current() != TokenKind::RightParenthesis {
        p.expect_token_kind(TokenKind::Comma, "Expected ','");
        expr(p);
    }

    p.expect_token_kind(TokenKind::RightParenthesis, "Expected ')'");

    p.end_node(call_begin, SyntaxKind::FunctionCallExpr)
}

#![cfg(test)]

use crate::featurez::parse::ParseEvent;
use crate::featurez::parse::Parser;
use crate::featurez::syntax::SyntaxKind;
use crate::featurez::syntax::SyntaxTree;
use crate::featurez::syntax::TextTreeSink;
use crate::featurez::tokenize;
use crate::featurez::StrTokenSource;
use crate::featurez::TokenKind;

// todo cover begin & end node logic.. additionally we need to add coverage to the tree builder
// I really should rationalize the distinctions between Syntax of Error vs. Syntax of Tombstone

#[test]
fn parser_current_returns_current_token_kind() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    assert_eq!(parser.current(), TokenKind::Plus);
    parser.token_if(TokenKind::Plus);
    assert_eq!(parser.current(), TokenKind::Minus);
}

#[test]
fn parser_current2_returns_current_and_next_token_kind() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    assert_eq!(parser.current2(), Some((TokenKind::Plus, TokenKind::Minus)));
    parser.token_if(TokenKind::Plus);
    assert_eq!(
        parser.current2(),
        Some((TokenKind::Minus, TokenKind::Slash))
    );
}

#[test]
fn parser_current_returns_token_kind_at_offset() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    assert_eq!(parser.nth(0), TokenKind::Plus);
    assert_eq!(parser.nth(1), TokenKind::Minus);
}

#[test]
fn parser_current_returns_end_of_file_at_end_of_tokens() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    assert_eq!(parser.nth(2), TokenKind::Slash);
    assert_eq!(parser.nth(3), TokenKind::EndOfFile);
}

#[test]
fn parser_current_returns_end_of_file_past_end_of_tokens() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    assert_eq!(parser.nth(30), TokenKind::EndOfFile);
}

#[test]
fn parser_token_if_produces_token_event_on_token_match() {
    let token_source = StrTokenSource::new(tokenize("+"));
    let mut parser = Parser::new(token_source);

    parser.token_if(TokenKind::Plus);

    let events = parser.end_parsing().events;
    let event = &events[1];

    assert_eq!(
        event,
        &ParseEvent::Token {
            kind: TokenKind::Plus,
            length: 1
        }
    );
}

#[test]
fn parser_token_if_produces_no_token_event_on_token_mismatch() {
    let token_source = StrTokenSource::new(tokenize("+"));
    let mut parser = Parser::new(token_source);

    parser.token_if(TokenKind::Minus);

    let events = parser.end_parsing().events;
    let token_event_count = events
        .into_iter()
        .filter(|e| match e {
            ParseEvent::Token { kind, length } => true,
            _ => false,
        })
        .count();

    assert_eq!(token_event_count, 0);
}

#[test]
fn parser_expect_token_kind_produces_token_event_on_token_match() {
    let token_source = StrTokenSource::new(tokenize("+"));
    let mut parser = Parser::new(token_source);

    parser.expect_token_kind(TokenKind::Plus, "Shouldn't see this");

    let events = parser.end_parsing().events;
    let event = &events[1];

    assert_eq!(
        event,
        &ParseEvent::Token {
            kind: TokenKind::Plus,
            length: 1
        }
    );
}

#[test]
fn parser_expect_token_kind_produces_error_syntax_node_on_mismatch() {
    let token_source = StrTokenSource::new(tokenize("+"));
    let mut parser = Parser::new(token_source);

    parser.expect_token_kind(TokenKind::Minus, "Expected '-'");

    let events = parser.end_parsing().events;

    let expected_error_node_start = &events[0];
    let expected_plus_token = &events[1];
    let expected_error_node_end = &events[2];

    assert_eq!(
        expected_error_node_start,
        &ParseEvent::BeginNode {
            kind: SyntaxKind::Error("Expected '-'"),
            is_forward_parent: false,
            forward_parent_offset: None
        }
    );
    assert_eq!(
        expected_plus_token,
        &ParseEvent::Token {
            kind: TokenKind::Plus,
            length: 1
        }
    );
    assert_eq!(expected_error_node_end, &ParseEvent::EndNode);
}

#[test]
fn parser_expect_token_kind_produces_tombstone_tokens_while_panicking() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    parser.expect_token_kind(TokenKind::Minus, "Expected '-'");
    parser.expect_token_kind(
        TokenKind::Plus,
        "This shouldn't error, but rather should be a tombstone token",
    );
    parser.expect_token_kind(TokenKind::Slash, "This should actually succeed");

    let events = parser.end_parsing().events;
    /*
        (error (Plus '+'))
        (TombStone '-')
        (Slash '/')
    */

    let expected_tombstone_token = &events[3];

    assert_eq!(
        expected_tombstone_token,
        &ParseEvent::Token {
            kind: TokenKind::TombStone,
            length: 1
        }
    );
}

#[test]
fn parser_expect_token_kind_recovers_from_panicking_if_expectation_is_met() {
    let token_source = StrTokenSource::new(tokenize("+-/+"));
    let mut parser = Parser::new(token_source);

    parser.expect_token_kind(TokenKind::Minus, "Expected '-'");
    parser.expect_token_kind(
        TokenKind::Plus,
        "This shouldn't error, but rather should be a tombstone token",
    );
    parser.expect_token_kind(TokenKind::Slash, "This should actually succeed");
    parser.expect_token_kind(TokenKind::Minus, "Second error");

    let events = parser.end_parsing().events;
    /*
        (error (Plus '+'))
        (TombStone '-')
        (Slash '/')
        (error (Plus '+'))
    */

    let expected_slash_token = &events[4];
    let expected_second_error_node = &events[5];

    assert_eq!(
        expected_slash_token,
        &ParseEvent::Token {
            kind: TokenKind::Slash,
            length: 1
        }
    );
    assert_eq!(
        expected_second_error_node,
        &ParseEvent::BeginNode {
            kind: SyntaxKind::Error("Second error"),
            is_forward_parent: false,
            forward_parent_offset: None
        }
    )
}

#[test]
fn parser_produces_error_node_if_tokens_remain_at_parsing_end() {
    let token_source = StrTokenSource::new(tokenize("+"));
    let mut parser = Parser::new(token_source);
    let events = parser.end_parsing().events;

    let expected_error_node_start = &events[0];
    let expected_plus_token = &events[1];
    let expected_error_node_end = &events[2];

    assert_eq!(
        expected_error_node_start,
        &ParseEvent::BeginNode {
            kind: SyntaxKind::Error("Unexpected text"),
            is_forward_parent: false,
            forward_parent_offset: None
        }
    );
    assert_eq!(
        expected_plus_token,
        &ParseEvent::Trivia {
            kind: TokenKind::Plus,
            length: 1
        }
    );
    assert_eq!(expected_error_node_end, &ParseEvent::EndNode);
}

#[test]
fn parser_begin_node_can_nest_nodes() {
    let token_source = StrTokenSource::new(tokenize("+-/"));
    let mut parser = Parser::new(token_source);

    let mut outer = parser.begin_node();
    parser.token_if(TokenKind::Plus);

    let mut inner = parser.begin_node();
    parser.token_if(TokenKind::Minus);
    parser.end_node(inner, SyntaxKind::UnaryExpr);

    parser.token_if(TokenKind::Slash);
    parser.end_node(outer, SyntaxKind::LiteralExpr);

    let events = parser.end_parsing().events;

    let expected_outer_node_start = &events[0];
    let expected_plus_token = &events[1];
    let expected_inner_node_start = &events[2];
    let expected_minus_token = &events[3];
    let expected_inner_node_end = &events[4];
    let expected_slash_token = &events[5];
    let expected_outer_node_end = &events[6];

    for event in events.iter() {
        println!("event: {:?}", event);
    }

    assert_eq!(
        expected_outer_node_start,
        &ParseEvent::BeginNode {
            kind: SyntaxKind::LiteralExpr,
            is_forward_parent: false,
            forward_parent_offset: None
        }
    );
    assert_eq!(
        expected_plus_token,
        &ParseEvent::Token {
            kind: TokenKind::Plus,
            length: 1
        }
    );

    assert_eq!(
        expected_inner_node_start,
        &ParseEvent::BeginNode {
            kind: SyntaxKind::UnaryExpr,
            is_forward_parent: false,
            forward_parent_offset: None
        }
    );
    assert_eq!(
        expected_minus_token,
        &ParseEvent::Token {
            kind: TokenKind::Minus,
            length: 1
        }
    );
    assert_eq!(expected_inner_node_end, &ParseEvent::EndNode);

    assert_eq!(
        expected_slash_token,
        &ParseEvent::Token {
            kind: TokenKind::Slash,
            length: 1
        }
    );
    assert_eq!(expected_outer_node_end, &ParseEvent::EndNode);
}

#[test]
fn parser_precede_node_can_precede_nodes() {
    let token_source = StrTokenSource::new(tokenize("1+2+3"));
    let mut parser = Parser::new(token_source);

    let mut outer = parser.begin_node();
    parser.token_if(TokenKind::IntegerLiteral);
    parser.token_if(TokenKind::Plus);
    parser.token_if(TokenKind::IntegerLiteral);
    let mut one_plus_two = parser.end_node(outer, SyntaxKind::BinaryExpr);

    let mut three_plus_three = parser.begin_node();
    parser.precede_node(&mut one_plus_two, &three_plus_three);
    parser.token_if(TokenKind::Plus);
    parser.token_if(TokenKind::IntegerLiteral);
    parser.end_node(three_plus_three, SyntaxKind::BinaryExpr);

    let tree = SyntaxTree::from_parser(&parser.end_parsing(), "1+2+3");
}

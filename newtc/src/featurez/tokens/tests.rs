#![cfg(test)]

use super::*;

use insta::assert_snapshot_matches;
use std::string::ToString;

use crate::featurez::syntax::TokenSource;

macro_rules! single_token_tests {
	($($name:ident: $value:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let (input_text, expected_token_kind) = $value;
			assert_single_token(input_text, expected_token_kind);
		}
	)*
	}
}

single_token_tests! {
    // single character tokens
    left_brace_token: ("{", TokenKind::LeftBrace),
    right_brace_token: ("}", TokenKind::RightBrace),
    left_parenthesis_token: ("(", TokenKind::LeftParenthesis),
    right_parenthesis_token: (")", TokenKind::RightParenthesis),
    left_bracket_token: ("[", TokenKind::LeftBracket),
    right_bracket_token: ("]", TokenKind::RightBracket),

    comma_token: (",", TokenKind::Comma),
    dot_token: (".", TokenKind::Dot),
    colon_token: (":", TokenKind::Colon),
    semicolon_token: (";", TokenKind::SemiColon),
    underscore_token: ("_", TokenKind::UnderScore),

    equals_token: ("=", TokenKind::Equals),
    plus_token: ("+", TokenKind::Plus),
    minus_token: ("-", TokenKind::Minus),
    star_token: ("*", TokenKind::Star),
    slash_token: ("/", TokenKind::Slash),

    greater_token: (">", TokenKind::Greater),
    less_token: ("<", TokenKind::Less),

    ampersand_token: ("&", TokenKind::Ampersand),
    pipe_token: ("|", TokenKind::Pipe),
    bang_token: ("!", TokenKind::Bang),

    // double character tokens
    equals_equals_token: ("==", TokenKind::EqualsEquals),
    greater_equals_token: (">=", TokenKind::GreaterEquals),
    less_equals_token: ("<=", TokenKind::LessEquals),
    ampersand_ampersand_token: ("&&", TokenKind::AmpersandAmpersand),
    pipe_pipe_token: ("||", TokenKind::PipePipe),

    // literals
    integer_literal_token: ("123", TokenKind::IntegerLiteral),
    float_literal_token: ("3.14", TokenKind::FloatLiteral),
    string_literal_token: ("\"Hello, world!\"", TokenKind::StringLiteral),
    glyph_literal_token: ("'c'", TokenKind::GlyphLiteral),

    // identifiers
    identifier_all_characters: ("_abc123", TokenKind::Identifier),
    identifier_alphanumeric: ("abc123", TokenKind::Identifier),
    identifier_alpha: ("abc", TokenKind::Identifier),

    // keywords
    fn_keyword: ("fn", TokenKind::Fn),
    return_keyword: ("return", TokenKind::Return),
    if_keyword: ("if", TokenKind::If),
    else_keyword: ("else", TokenKind::Else),
    for_keyword: ("for", TokenKind::For),

    in_keyword: ("in", TokenKind::In),
    while_keyword: ("while", TokenKind::While),
    let_keyword: ("let", TokenKind::Let),
    true_keyword: ("true", TokenKind::True),
    false_keyword: ("false", TokenKind::False),
}

macro_rules! token_sequence_tests {
	($($name:ident: $value:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let input_text = $value;
			let tokens = tokenize(input_text)
				.iter()
				.map(|t| t.to_string())
				.collect::<Vec<String>>();

			let document = format!("{}\n\n===\n\n{}", input_text, tokens.join("\n"));

			assert_snapshot_matches!(stringify!($name), document);
		}
	)*
	}
}

token_sequence_tests! {
    identifiers_can_start_with_underscore: "_foo123",
    identifiers_can_have_underscores_in_middle: "foo_123",
    identifiers_can_not_start_with_numbers: "123foo",
    identifiers_can_not_be_just_underscores: "_",
    tombstones_do_not_stop_tokenizing: "foo`bar`fizz",
    comment_lines_consume_whole_line: "foo//not identifier`token\n123",
    equals_equals_equals: "===",
    greater_equals_equals: ">==",
    less_equals_equals: "<==",
    ampersand_ampersand_ampersand: "&&&",
    pipe_pipe_pipe: "|||",
    literals:
"1234
3.14
'c'
\"foo\"",
    identifiers:
"_validIdentifier123
_456validIdentifier
456badIdentifier
seperate identifiers",
    keywords:
"if else 
let 
fn
while
for in
return
module
true
false",
    operators:
"// Math operators
let x=1+2-3/4*5+(6);
// Logic operators
let y=true&&false||!true;
// Comparison operators 
let z=2==2;
let a=4<2;
let b=4>2;
let c=4<=2;
let c=4>=2;
// Assignment operator, used throughout
let d=2;",
    starting_whitespace: r#"
    let x=1+2;"#,
}

fn assert_single_token(value: &str, expected_type: TokenKind) {
    let tokens = tokenize(value);

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token_kind(), expected_type);
    assert_eq!(tokens[1].token_kind(), TokenKind::EndOfFile);
}

fn assert_token_sequence(value: &str, expected_tokens: &[TokenKind]) {
    use std::cmp::min;

    let actual_tokens = tokenize(value);
    let max_safe_length = min(actual_tokens.len(), expected_tokens.len());

    for index in 0..max_safe_length {
        assert_eq!(actual_tokens[index].token_kind(), expected_tokens[index]);
    }

    assert_eq!(actual_tokens.len(), expected_tokens.len());
}

#[test]
fn token_source_token_kind_gets_type_at_position() {
    let source = "2+2==4;";
    let tokens = tokenize(source);
    let token_source = StrTokenSource::new(tokens);

    assert_eq!(token_source.token_kind(0), TokenKind::IntegerLiteral);
    assert_eq!(token_source.token_kind(3), TokenKind::EqualsEquals);
}

#[test]
fn token_source_token_gets_token_at_position() {
    let source = "2+2==4;";
    let tokens = tokenize(source);
    let token_source = StrTokenSource::new(tokens);

    assert_eq!(
        token_source.token(0).token_kind(),
        TokenKind::IntegerLiteral
    );
    assert_eq!(token_source.token(0).lexeme_length(), 1);
    assert_eq!(token_source.token(3).token_kind(), TokenKind::EqualsEquals);
    assert_eq!(token_source.token(3).lexeme_length(), 2);
}

#[test]
fn token_source_token_kind_gets_end_of_file_when_out_of_bounds() {
    let source = "2+2==4;";
    let tokens = tokenize(source);
    let token_source = StrTokenSource::new(tokens);

    assert_eq!(token_source.token_kind(5), TokenKind::SemiColon);
    assert_eq!(token_source.token_kind(6), TokenKind::EndOfFile);
    assert_eq!(token_source.token_kind(10), TokenKind::EndOfFile);
}

#[test]
fn token_source_token_gets_end_of_file_when_out_of_bounds() {
    let source = "2+2==4;";
    let tokens = tokenize(source);
    let token_source = StrTokenSource::new(tokens);

    assert_eq!(token_source.token(5).token_kind(), TokenKind::SemiColon);
    assert_eq!(token_source.token(6).token_kind(), TokenKind::EndOfFile);
    assert_eq!(token_source.token(10).token_kind(), TokenKind::EndOfFile);
}

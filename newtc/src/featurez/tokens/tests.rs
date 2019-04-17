#![cfg(test)]

use crate::featurez::tokens::{tokenize, Token, TokenType};

macro_rules! single_token_tests {
	($($name:ident: $value:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let (input_text, expected_token_type) = $value;
			assert_single_token(input_text, expected_token_type);
		}
	)*
	}
}

single_token_tests! {
	// single character tokens
	left_brace_token: ("{", TokenType::LeftBrace),
	right_brace_token: ("}", TokenType::RightBrace),
	left_parenthesis_token: ("(", TokenType::LeftParenthesis),
	right_parenthesis_token: (")", TokenType::RightParenthesis),
	left_bracket_token: ("[", TokenType::LeftBracket),
	right_bracket_token: ("]", TokenType::RightBracket),

	comma_token: (",", TokenType::Comma),
	dot_token: (".", TokenType::Dot),
	colon_token: (":", TokenType::Colon),
	semicolon_token: (";", TokenType::SemiColon),
	underscore_token: ("_", TokenType::UnderScore),

	equals_token: ("=", TokenType::Equals),
	plus_token: ("+", TokenType::Plus),
	minus_token: ("-", TokenType::Minus),
	star_token: ("*", TokenType::Star),
	slash_token: ("/", TokenType::Slash),

	greater_token: (">", TokenType::Greater),
	less_token: ("<", TokenType::Less),

	ampersand_token: ("&", TokenType::Ampersand),
	pipe_token: ("|", TokenType::Pipe),
	bang_token: ("!", TokenType::Bang),
	
	// double character tokens
	equals_equals_token: ("==", TokenType::EqualsEquals),
	greater_equals_token: (">=", TokenType::GreaterEquals),
	less_equals_token: ("<=", TokenType::LessEquals),
	ampersand_ampersand_token: ("&&", TokenType::AmpersandAmpersand),
	pipe_pipe_token: ("||", TokenType::PipePipe),
	
	// literals
	integer_literal_token: ("123", TokenType::IntegerLiteral),
	float_literal_token: ("3.14", TokenType::FloatLiteral),
	string_literal_token: ("\"Hello, world!\"", TokenType::StringLiteral),
	glyph_literal_token: ("'c'", TokenType::GlyphLiteral),
	
	// identifiers
	identifier_all_characters: ("_abc123", TokenType::Identifier),
	identifier_alphanumeric: ("abc123", TokenType::Identifier),
	identifier_alpha: ("abc", TokenType::Identifier),
	
	// keywords
	fn_keyword: ("fn", TokenType::Fn),
	return_keyword: ("return", TokenType::Return),
	if_keyword: ("if", TokenType::If),
	else_keyword: ("else", TokenType::Else),
	for_keyword: ("for", TokenType::For),
	
	in_keyword: ("in", TokenType::In),
	while_keyword: ("while", TokenType::While),
	let_keyword: ("let", TokenType::Let),
	true_keyword: ("true", TokenType::True),
	false_keyword: ("false", TokenType::False),
}
/*
macro_rules! token_sequence_tests {
	($($name:ident: $value:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let (input_text, expected_token_sequence) = $value;
			assert_token_sequence(input_text, &expected_token_sequence);
		}
	)*
	}
}

token_sequence_tests! {
	identifiers_can_start_with_underscore: ("_foo123", [
		TokenType::Identifier,
		TokenType::EndOfFile
	]),
	
	identifiers_can_have_underscores_in_middle: ("foo_123", [
		TokenType::Identifier, 
		TokenType::EndOfFile
	]),
	
	identifiers_can_not_start_with_numbers: ("123foo", [
		TokenType::IntegerLiteral, 
		TokenType::Identifier, 
		TokenType::EndOfFile
	]),
	
	identifiers_can_not_be_just_underscores: ("_", [
		TokenType::UnderScore, 
		TokenType::EndOfFile
	]),
													
	tombstones_do_not_stop_tokenizing: ("foo`bar`fizz", [
		TokenType::Identifier, 
		TokenType::TombStone,
		TokenType::Identifier, 
		TokenType::TombStone,
		TokenType::Identifier, 
		TokenType::EndOfFile
	]),
														
	tombstones_which_are_adjacent_are_merged: ("foo``fizz", [
		TokenType::Identifier,
		TokenType::TombStone,
		TokenType::Identifier, 
		TokenType::EndOfFile
	]),
														
	comment_lines_consume_whole_line: ("foo//not identifier`token\n123", [
		TokenType::Identifier,
		TokenType::CommentLine,
		TokenType::WhiteSpace,
		TokenType::IntegerLiteral,
		TokenType::EndOfFile
	]),
	
	equals_equals_equals: ("===", [
		TokenType::EqualsEquals,
		TokenType::Equals,
		TokenType::EndOfFile
	]),
	
	greater_equals_equals: (">==", [
		TokenType::GreaterEquals,
		TokenType::Equals,
		TokenType::EndOfFile
	]),
	
	less_equals_equals: ("<==", [
		TokenType::LessEquals,
		TokenType::Equals,
		TokenType::EndOfFile
	]),
	
	ampersand_ampersand_ampersand: ("&&&", [
		TokenType::AmpersandAmpersand,
		TokenType::Ampersand,
		TokenType::EndOfFile
	]),
	
	pipe_pipe_pipe: ("|||", [
		TokenType::PipePipe,
		TokenType::Pipe,
		TokenType::EndOfFile
	]),
}
*/
fn assert_single_token(value: &str, expected_type: TokenType) {
	let tokens = tokenize(value);

	assert_eq!(tokens.len(), 2);
	assert_eq!(tokens[0].token_type, expected_type);
	assert_eq!(tokens[1].token_type, TokenType::EndOfFile);
}

fn assert_token_sequence(value: &str, expected_tokens: &[TokenType]) {
	use std::cmp::min;

	let actual_tokens = tokenize(value);
	let max_safe_length = min(actual_tokens.len(), expected_tokens.len());

	for index in 0..max_safe_length {
		assert_eq!(actual_tokens[index].token_type, expected_tokens[index]);
	}

	assert_eq!(actual_tokens.len(), expected_tokens.len());
}
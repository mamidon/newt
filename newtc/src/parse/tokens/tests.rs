#![cfg(test)]

use crate::parse::tokens::{tokenize, Token, TokenType};

macro_rules! single_character_token_tests {
	($($name:ident: $value:expr,)*) => {
	$(
		#[test]
		fn $name() {
			let (input_text, expected_token_type) = $value;
			assert_single_character_token(input_text, expected_token_type);
		}
	)*
	}
}

single_character_token_tests! {
	left_brace_test: ("{", TokenType::LeftBrace),
	right_brace_test: ("}", TokenType::RightBrace),
	left_parenthesis_test: ("(", TokenType::LeftParenthesis),
	right_parenthesis_test: (")", TokenType::RightParenthesis),
	left_bracket_test: ("[", TokenType::LeftBracket),
	right_bracket_test: ("]", TokenType::RightBracket),


	comma_test: (",", TokenType::Comma),
	dot_test: (".", TokenType::Dot),
	colon_test: (":", TokenType::Colon),
	semicolon_test: (";", TokenType::Semicolon),
	underscore_test: ("_", TokenType::Underscore),


	equals_test: ("=", TokenType::Equals),
	plus_test: ("+", TokenType::Plus),
	minus_test: ("-", TokenType::Minus),
	star_test: ("*", TokenType::Star),
	slash_test: ("/", TokenType::Slash),


	greater_test: (">", TokenType::Greater),
	less_test: ("<", TokenType::Less),

	ampersand_test: ("&", TokenType::Ampersand),
	pipe_test: ("|", TokenType::Pipe),
	bang_test: ("!", TokenType::Bang),
}

fn assert_single_character_token(value: &str, expected_type: TokenType) 
{
	let tokens = tokenize(value);
	
	assert_eq!(tokens.len(), 1);
	assert_eq!(tokens[0].token_type, expected_type);
	//assert_eq!(tokens[1].token_type, TokenType::EndOfFile);
}
	

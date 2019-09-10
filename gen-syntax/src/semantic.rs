use crate::tokens::{Token, TokenKind};
use crate::parse::{Production, ParseError, ParseErrorKind};
use std::collections::HashSet;


pub fn validate(root: &Production, source: &str) -> Vec<ParseError> {

	let mut errors: Vec<ParseError> = vec![];

	errors.extend(check_undefined_symbols(root, source));

	errors
}

fn check_undefined_symbols(root: &Production, source: &str) -> Vec<ParseError> {
	let mut errors: Vec<ParseError> = vec![];
	let mut defined_symbols: HashSet<String> = HashSet::new();

	for rule in root.iter() {
		if let Production::Rule { name: token, production: _ } = rule {
			let symbol = source[token.offset..token.offset + token.length].to_string();
			if !defined_symbols.contains(&symbol) {
				defined_symbols.insert(symbol);
			} else {
				errors.push(ParseError::new(*token, ParseErrorKind::DuplicateSymbol { symbol }));
			}
		}
	}

	for production in root.iter() {
		match production {
			Production::Identifier { rule_name, member_name } => {
				let symbol = source[rule_name.offset..rule_name.offset + rule_name.length].to_string();

				if !defined_symbols.contains(&symbol) {
					errors.push(ParseError::new(*rule_name, ParseErrorKind::UndefinedSymbol { symbol }));
				}
			},
			_ => {}
		}
	}

	errors
}

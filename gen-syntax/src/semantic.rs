use crate::tokens::{Token, TokenKind};
use crate::parse::{Production, ParseError, ParseErrorKind};
use std::collections::{HashSet, HashMap};

struct SemanticsContext<'a> {
	source: &'a str,
	symbols: HashMap<&'a str, Symbol>
}

impl<'a> SemanticsContext<'a> {
	fn new(source: &'a str) -> SemanticsContext<'a> {
		SemanticsContext {
			source,
			symbols: HashMap::new()
		}
	}

	fn define_symbol(&mut self, token: Token) -> Result<Symbol, ParseError> {
		let lexeme = self.lexeme(token);
		if !self.symbols.contains_key(lexeme) {
			let symbol = Symbol::new();
			self.symbols.insert(lexeme, symbol);
			Ok(symbol)
		} else {
			Err(ParseError::new(token, ParseErrorKind::DuplicateSymbol { symbol: lexeme.to_string() }))
		}
	}

	fn get_symbol(&self, token: Token) -> Option<&Symbol> {
		self.symbols.get(self.lexeme(token))
	}

	fn lexeme(&self, token: Token) -> &'a str {
		&self.source[token.offset..token.offset + token.length]
	}
}

#[derive(Copy, Clone)]
struct Symbol;

impl Symbol {
	fn new() -> Symbol { Symbol {} }
}

pub fn validate_semantics(root: &Production, source: &str) -> Vec<ParseError> {
	let mut context = SemanticsContext::new(source);
	let mut errors: Vec<ParseError> = vec![];

	errors.extend(define_symbols(&mut context, root));
	errors.extend(check_undefined_symbols(&context, root));
	errors.extend(check_ambiguous_pipes(&context, root));

	errors
}

fn define_symbols(context: &mut SemanticsContext, root: &Production) -> Vec<ParseError> {
	let mut errors: Vec<ParseError> = vec![];

	for rule in root.iter() {
		if let Production::Rule { name: token, production: _ } = rule {
			match context.define_symbol(*token) {
				Ok(symbol) => {},
				Err(error) => errors.push(error)
			}
		}
	}

	errors
}

fn check_undefined_symbols(context: &SemanticsContext, root: &Production) -> Vec<ParseError> {
	let mut errors: Vec<ParseError> = vec![];

	for production in root.iter() {
		if let Production::Identifier { rule_name, member_name: _ } = production {
			if context.get_symbol(*rule_name).is_none() {
				errors.push(ParseError::new(*rule_name, ParseErrorKind::UndefinedSymbol {
					symbol: context.lexeme(*rule_name).to_string()
				}));
			}
		}
	}

	errors
}

fn check_ambiguous_pipes(context: &SemanticsContext, root: &Production) -> Vec<ParseError> {
	let mut errors: Vec<ParseError> = vec![];

	for production in root.iter() {
		if let Production::Pipe(options) = production {
			for option in options.iter() {
				let identifier_tokens: Vec<Token> = option.iter()
					.filter_map(|p|
						match p {
							Production::Identifier { rule_name, member_name} => Some(*rule_name),
							_ => None
					})
					.collect();

				let unique_identifiers: HashSet<String> = identifier_tokens.iter()
					.map(|t| context.lexeme(*t).to_string())
					.collect();

				if unique_identifiers.len() > 1 {
					let error_kind = ParseErrorKind::AmbiguousPipe;

					errors.push(ParseError::new(*identifier_tokens.first().unwrap(), error_kind))
				}
			}
		}
	}

	errors
}

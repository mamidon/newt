#![allow(dead_code)]

mod tokens;

use tokens::{tokenize};
use std::io::{self, Read, Write};



fn main() -> io::Result<()> {
	let mut stdin = io::stdin();
	let stdout = io::stdout();
	let mut buffer = String::new();

	stdin.read_to_string(&mut buffer)?;

	let tokens = tokenize(&buffer);

	let mut lock = stdout.lock();
	for token in tokens {
		writeln!(lock, "{:?}", token)?;
	}

	Ok(())
}




struct ParseError;
struct GrammarRule {
	name: String,
	production: GrammarProduction
}
struct RuleIdentifier {
	rule_name: String,
	member_name: Option<String>
}

enum OperatorKind {
	Plus,
	Star,
	Grouping
}

enum Syntax {
	Error(ParseError),

	Rule(GrammarRule),
	Production(GrammarProduction)
}

enum GrammarProduction {
	Plus(RuleIdentifier),
	Star(RuleIdentifier),
	Grouping(Box<GrammarProduction>),
	Pipe(Box<GrammarProduction>, Box<GrammarProduction>),
	RuleIdentifier(RuleIdentifier)
}

/*

Expr => UnaryExpr+ | BinaryExpr*
BinaryExpr => Expr[lhs] Token[op] Expr[rhs]
*/



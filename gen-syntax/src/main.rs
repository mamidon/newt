#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod tokens;
mod parse;
mod semantic;

use tokens::{tokenize};
use parse::{parse, ErrorReport};
use semantic::validate;

use std::io::{self, Read};
use crate::parse::{Production, ParseError};

extern crate ansi_term;

/*
TODO -- actually deliver value now

1. If any errors are found, print them out and stop
2. Gather up the distinct set of node types to create the top level SyntaxKind enum
2. Wherever there is a pipe operator, create a sub-kind.  e.g. Expr & Stmt
-- it is illegal for any there to be multiple identifiers in a pipe branch, we won't
-- know what to name it's relevant kind et al
3. Wherever there is a * emit no methods

TODO -- later allow for the specification of a set of tokens e.g. "{" "(" "." etc
TODO -- any tokens referenced not in the set are treated as undefined symbols, but otherwise
TODO -- they have no effect on the output
TODO -- once that's done, dogfood on this crate
*/
fn main() -> Result<(), ErrorReport> {
	let outcome = main_core();

	match outcome {
		Ok(output) => println!("{:#?}", output),
		Err(errors) => errors.iter().for_each(|e| println!("{}", e))
	}

	Ok(())
}

fn main_core() -> Result<Production, Vec<ErrorReport>> {
	let mut buffer = String::new();

	io::stdin()
		.read_to_string(&mut buffer)
		.map_err(map_io_error_to_reports)?;

	let tokens = tokenize(&buffer);
	let parsing = parse(tokens)
		.map_err(|errors| map_parse_errors_to_reports(errors, &buffer))?;

	let reports: Vec<ErrorReport> = validate(&parsing, &buffer)
		.iter()
		.map(|e| ErrorReport::from_parse_error(e, &buffer))
		.collect();

	if !reports.is_empty() {
		Err(reports)
	} else {
		Ok(parsing)
	}
}

fn map_io_error_to_reports(error: std::io::Error) -> Vec<ErrorReport> {
	vec![ErrorReport::from_io_error(error)]
}

fn map_parse_errors_to_reports(errors: Vec<ParseError>, source: &str) -> Vec<ErrorReport> {
	errors.iter()
		.map(|parse_error| ErrorReport::from_parse_error(parse_error, &source))
		.collect()
}
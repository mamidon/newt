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
fn main() -> io::Result<()> {
	let mut buffer = String::new();

	io::stdin().read_to_string(&mut buffer)?;

	let tokens = tokenize(&buffer);
	let parsing = parse(tokens);

	match parsing {
		Ok(root) => {
			let reports: Vec<ErrorReport> = validate(&root, &buffer)
				.iter()
				.map(|e| ErrorReport::from_parse_error(e, &buffer))
				.collect();

			for report in reports.iter() {
				println!("{}\n", report);
			}

			if reports.is_empty() {
				println!("{:#?}", root);
			}
		},
		Err(errors) => {
			let reports: Vec<ErrorReport> = errors.iter()
				.map(|e| ErrorReport::from_parse_error(e, &buffer))
				.collect();

			for report in reports {
				println!("{}\n", report);
			}
		}
	}

	Ok(())
}
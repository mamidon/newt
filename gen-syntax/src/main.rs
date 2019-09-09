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

fn main() -> io::Result<()> {
	let mut buffer = String::new();

	io::stdin().read_to_string(&mut buffer)?;

	let tokens = tokenize(&buffer);
	let parsing = parse(tokens);

	match parsing {
		Ok(root) => {
			validate(&root, &buffer).unwrap();
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
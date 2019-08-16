#![allow(dead_code)]

mod tokens;
mod parse;

use tokens::{tokenize};
use parse::parse;

use std::io::{self, Read, Write};



fn main() -> io::Result<()> {
	let mut stdin = io::stdin();
	let stdout = io::stdout();
	let mut buffer = String::new();

	stdin.read_to_string(&mut buffer)?;

	let tokens = tokenize(&buffer);

	let mut lock = stdout.lock();

	writeln!(lock, "{:#?}", tokens)?;

	let parsing = parse(tokens);

	writeln!(lock, "{:#?}", parsing)?;

	Ok(())
}




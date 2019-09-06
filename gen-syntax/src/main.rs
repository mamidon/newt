#![allow(dead_code)]

mod tokens;
mod parse;
mod semantic;

use tokens::{tokenize};
use parse::{parse, ParseError, ParseErrorKind};
use semantic::validate;

use std::io::{self, Read};

extern crate ansi_term;
use ansi_term::Color::Red;
use ansi_term::Style;

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
				println!("{}: {}",
				         Red.paint(format!("{}", report.line_number)),
				         Red.paint(report.message));
				println!("{}{}{}",
				         report.leading_context,
				         Red.underline().paint(report.failing_context),
				         report.trailing_context);
			}
		}
	}

	Ok(())
}

struct ErrorReport<'a> {
	message: String,
	line_number: usize,
	leading_context: &'a str,
	failing_context: &'a str,
	trailing_context: &'a str
}

impl<'a> ErrorReport<'a> {
	fn from_parse_error(error: &ParseError, source: &'a str) -> ErrorReport<'a> {
		let message: String = match error.kind {
			ParseErrorKind::UnexpectedToken { expected, actual } => {
				format!("Expected {:?}, but found {:?}.", expected, actual)
			},
			ParseErrorKind::MissingSyntax { message } => message.to_string(),
		};

		let from = error.location.offset;
		let to = error.location.length + from;
		let line_number = source[..from].lines().count();
		let leading_context_start = from - source[..from]
			.chars()
			.rev()
			.take_while(|c| *c != '\n')
			.take_while(|c| *c != '\n')
			.count();
		let trailing_context_end = to + source[to..].chars().rev()
			.take_while(|c| *c != '\n')
			.take_while(|c| *c != '\n')
			.count();

		ErrorReport {
			message,
			line_number,
			leading_context: &source[leading_context_start..from],
			failing_context: &source[from..to],
			trailing_context: &source[to..trailing_context_end]
		}
	}
}

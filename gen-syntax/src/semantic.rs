use crate::tokens::{Token, TokenKind};
use crate::parse::{Production, ParseError};


pub fn validate(root: &Production, source: &str) -> Result<(), Vec<ParseError>> {

	println!("{:#?}", root);

	Ok(())
}

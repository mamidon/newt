#[derive(Debug, Clone)]
pub struct TokenizationError {
	line_number: usize,
	preceding_chars: usize,
	message: &'static str
}

#[derive(Debug, Clone)]
pub enum Token {
	Error(TokenizationError),
	EndOfFile,
	Trivia(String),
	Identifier(String),
	Arrow,
	LeftParenthesis,
	RightParenthesis,
	LeftBracket,
	RightBracket,
	Quoted(String),
	Pipe,
	Star,
	Plus
}

struct Characters<'a> {
	source: &'a str,
	lines_consumed: usize,
	chars_consumed: usize
}

impl<'a> Characters<'a> {
	pub fn new(source: &str) -> Characters {
		Characters {
			source,
			lines_consumed: 0,
			chars_consumed: 0
		}
	}

	pub fn peek(&self) -> Option<char> {
		self.peek_nth(0)
	}

	pub fn peek_nth(&self, offset: usize) -> Option<char> {
		self.source.chars().nth(offset)
	}

	pub fn lines_consumed(&self) -> usize {
		self.lines_consumed
	}

	pub fn chars_consumed(&self) -> usize {
		self.chars_consumed
	}

	pub fn consume(&mut self) {
		if let Some(c) = self.peek() {
			if c == '\n' {
				self.lines_consumed += 1;
			}

			self.source = &self.source[1..];
		}
	}
}

pub fn tokenize(source: &str) -> Vec<Token> {
	let mut cursor = Characters::new(source);
	let mut tokens: Vec<Token> = vec![];

	while cursor.peek().is_some() {
		tokens.push(next_token(&mut cursor));
	}
	tokens.push(Token::EndOfFile);

	tokens
}

fn next_token(cursor: &mut Characters) -> Token {
	let character = cursor.peek().expect("tokenize should've handled EndOfFile");
	cursor.consume();

	match character {
		'=' => next_token_arrow(cursor),
		'(' => Token::LeftParenthesis,
		')' => Token::RightParenthesis,
		'[' => Token::LeftBracket,
		']' => Token::RightBracket,
		'|' => Token::Pipe,
		'*' => Token::Star,
		'+' => Token::Plus,
		'\'' => next_token_quoted(cursor),
		c if c.is_alphabetic() => next_token_identifier(cursor, c),
		c if c.is_whitespace() => next_token_trivia(cursor, c),
		c => next_token_error(cursor, c)
	}

}

fn next_token_arrow(cursor: &mut Characters) -> Token {
	match cursor.peek() {
		Some('>') => {
			cursor.consume();

			Token::Arrow
		},
		_ => Token::Error(TokenizationError {
			line_number: cursor.lines_consumed(),
			preceding_chars: cursor.chars_consumed(),
			message: "Expected a '>' in '=>'"
		})
	}
}

fn next_token_identifier(cursor: &mut Characters, first: char) -> Token {
	let mut identifier = first.to_string();

	while let Some(c) = cursor.peek().filter(|c| c.is_alphabetic()) {
		identifier.push(c);
		cursor.consume();
	}

	Token::Identifier(identifier)
}

fn next_token_quoted(cursor: &mut Characters) -> Token {
	let mut quoted = String::new();

	while let Some(c) = cursor.peek() {
		cursor.consume();

		if c != '\'' {
			quoted.push(c);
		} else {
			return Token::Quoted(quoted);
		}
	}

	return Token::Error(TokenizationError {
		line_number: cursor.lines_consumed(),
		preceding_chars: cursor.chars_consumed(),
		message: "Expected a closing ', but found end of file"
	});
}

fn next_token_trivia(cursor: &mut Characters, first: char) -> Token {
	let mut trivia = first.to_string();

	while let Some(c) = cursor.peek().filter(|c| c.is_whitespace()) {
		trivia.push(c);
		cursor.consume();
	}

	Token::Trivia(trivia)
}

fn next_token_error(cursor: &mut Characters, _fault: char) -> Token {
	Token::Error(TokenizationError {
		line_number: cursor.lines_consumed(),
		preceding_chars: cursor.chars_consumed(),
		message: "Unexpected character '{}'"
	})
}
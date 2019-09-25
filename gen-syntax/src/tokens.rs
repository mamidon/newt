#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TokenKind {
    Error,
    EndOfFile,
    Trivia,
    SemiColon,
    Identifier,
    Arrow,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    Quoted,
    Pipe,
    Star,
    Plus,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: usize,
    pub length: usize,
}

impl Token {
    pub fn new(kind: TokenKind, offset: usize, length: usize) -> Token {
        Token {
            kind,
            offset,
            length,
        }
    }

    pub fn is_trivia(&self) -> bool {
        self.kind == TokenKind::Trivia
    }

    pub fn is_error(&self) -> bool {
        self.kind == TokenKind::Error
    }

    pub fn is_identifier(&self) -> bool {
        self.kind == TokenKind::Identifier
    }

    pub fn is_quoted(&self) -> bool {
        self.kind == TokenKind::Quoted
    }
}

struct Characters<'a> {
    source: &'a str,
    chars_consumed: usize,
}

impl<'a> Characters<'a> {
    pub fn new(source: &str) -> Characters {
        Characters {
            source,
            chars_consumed: 0,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_nth(0)
    }

    pub fn peek_nth(&self, offset: usize) -> Option<char> {
        self.source.chars().nth(offset)
    }

    pub fn chars_consumed(&self) -> usize {
        self.chars_consumed
    }

    pub fn consume(&mut self) {
        if let Some(_) = self.peek() {
            self.chars_consumed += 1;
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
    tokens.push(Token::new(TokenKind::EndOfFile, cursor.chars_consumed, 0));

    tokens
}

fn next_token(cursor: &mut Characters) -> Token {
    let character = cursor.peek().expect("tokenize should've handled EndOfFile");
    let offset = cursor.chars_consumed;

    cursor.consume();

    match character {
        '=' => next_token_arrow(cursor, offset),
        '(' => Token::new(TokenKind::LeftParenthesis, offset, 1),
        ')' => Token::new(TokenKind::RightParenthesis, offset, 1),
        '[' => Token::new(TokenKind::LeftBracket, offset, 1),
        ']' => Token::new(TokenKind::RightBracket, offset, 1),
        '|' => Token::new(TokenKind::Pipe, offset, 1),
        '*' => Token::new(TokenKind::Star, offset, 1),
        '+' => Token::new(TokenKind::Plus, offset, 1),
        ';' => Token::new(TokenKind::SemiColon, offset, 1),
        '\'' => next_token_quoted(cursor),
        c if c.is_alphabetic() => next_token_identifier(cursor, offset),
        c if c.is_whitespace() => next_token_trivia(cursor, offset),
        _ => next_token_error(cursor, offset),
    }
}

fn next_token_arrow(cursor: &mut Characters, start_offset: usize) -> Token {
    match cursor.peek() {
        Some('>') => {
            cursor.consume();

            Token::new(
                TokenKind::Arrow,
                start_offset,
                cursor.chars_consumed - start_offset,
            )
        }
        _ => Token::new(
            TokenKind::Error,
            start_offset,
            cursor.chars_consumed - start_offset,
        ),
    }
}

fn next_token_identifier(cursor: &mut Characters, start_offset: usize) -> Token {
    while let Some(_) = cursor.peek().filter(|c| c.is_alphabetic()) {
        cursor.consume();
    }

    Token::new(
        TokenKind::Identifier,
        start_offset,
        cursor.chars_consumed - start_offset,
    )
}

fn next_token_quoted(cursor: &mut Characters) -> Token {
    let offset = cursor.chars_consumed - 1;

    while let Some(c) = cursor.peek() {
        cursor.consume();

        if c == '\'' {
            return Token::new(TokenKind::Quoted, offset, cursor.chars_consumed - offset);
        }
    }

    Token::new(TokenKind::Error, offset, cursor.chars_consumed - offset)
}

fn next_token_trivia(cursor: &mut Characters, start_offset: usize) -> Token {
    while let Some(_) = cursor.peek().filter(|c| c.is_whitespace()) {
        cursor.consume();
    }

    Token::new(
        TokenKind::Trivia,
        start_offset,
        cursor.chars_consumed - start_offset,
    )
}

fn next_token_error(cursor: &mut Characters, start_offset: usize) -> Token {
    Token::new(
        TokenKind::Error,
        start_offset,
        cursor.chars_consumed - start_offset,
    )
}

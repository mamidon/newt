use crate::tokens::{Token, TokenKind};
use ansi_term::Color::Red;
use std::cmp::{max, min};
use std::fmt::{Debug, Display, Error, Formatter};
use std::net::ToSocketAddrs;

struct Tokens<'a> {
    source: &'a [Token],
    tokens_consumed: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(source: &[Token]) -> Tokens {
        Tokens {
            source,
            tokens_consumed: 0,
        }
    }

    pub fn peek(&self) -> Token {
        self.peek_nth(0)
    }

    pub fn peek_nth(&self, offset: usize) -> Token {
        if offset < self.source.len() {
            *self.source.iter().nth(offset).unwrap()
        } else {
            Token::new(TokenKind::EndOfFile, 0, 0)
        }
    }

    pub fn tokens_consumed(&self) -> usize {
        self.tokens_consumed
    }

    pub fn consume(&mut self) {
        if self.source.len() >= 1 {
            self.tokens_consumed += 1;
            self.source = &self.source[1..];
        }
    }

    pub fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        let actual = self.peek();
        self.consume();

        if actual.kind == expected {
            Ok(actual)
        } else {
            Err(ParseError::new(
                actual,
                ParseErrorKind::UnexpectedToken {
                    expected,
                    actual: actual.kind,
                },
            ))
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub(crate) location: Token,
    pub(crate) kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedToken {
        expected: TokenKind,
        actual: TokenKind,
    },
    MissingSyntax {
        message: &'static str,
    },
    UndefinedSymbol {
        symbol: String,
    },
    DuplicateSymbol {
        symbol: String,
    },
    AmbiguousPipe,
}

impl ParseError {
    pub fn new(location: Token, kind: ParseErrorKind) -> ParseError {
        ParseError { kind, location }
    }
}

pub struct ErrorReport {
    pub message: String,
    pub line_number: usize,
    lines: Vec<ErrorReportLine>,
}

struct ErrorReportLine {
    line_number: usize,
    line: String,
    error_span: Option<(usize, usize)>,
}

impl ErrorReport {
    pub fn from_parse_error(error: &ParseError, source: &str) -> ErrorReport {
        let message: String = match &error.kind {
            ParseErrorKind::UnexpectedToken { expected, actual } => {
                format!("Expected {:?}, but found {:?}.", expected, actual)
            }
            ParseErrorKind::MissingSyntax { message } => message.to_string(),
            ParseErrorKind::DuplicateSymbol { symbol } => {
                format!("Duplicate symbol '{:?}' detected", &symbol)
            }
            ParseErrorKind::UndefinedSymbol { symbol } => {
                format!("Undefined symbol '{:?}' detected", &symbol)
            }
            ParseErrorKind::AmbiguousPipe => format!("Ambiguous pipe statement are not allowed"),
        };
        let context_lines = 1;

        let from = error.location.offset;
        let to = error.location.length + from;
        let lines_preceding = source[..from].chars().filter(|c| *c == '\n').count();
        let lines_to_skip = lines_preceding - min(lines_preceding, context_lines);
        let lines: Vec<ErrorReportLine> = source
            .lines()
            .enumerate()
            .scan(0, |chars, tuple| {
                let line_span = (*chars, *chars + tuple.1.len());
                *chars = *chars + tuple.1.len() + 1;

                Some(ErrorReportLine {
                    line_number: tuple.0 + 1,
                    line: tuple.1.to_string(),
                    error_span: if line_span.1 < from || line_span.0 > to {
                        None
                    } else {
                        Some((
                            max(line_span.0, from) - line_span.0,
                            min(line_span.1, to) - line_span.0,
                        ))
                    },
                })
            })
            .skip(lines_to_skip)
            .take(context_lines * 2 + 1)
            .collect();

        ErrorReport {
            message,
            line_number: lines_preceding + 1,
            lines,
        }
    }

    pub fn from_io_error(error: std::io::Error) -> ErrorReport {
        ErrorReport {
            line_number: 0,
            lines: vec![],
            message: format!("{:?}", error.kind()),
        }
    }
}

impl From<std::io::Error> for ErrorReport {
    fn from(error: std::io::Error) -> Self {
        ErrorReport::from_io_error(error)
    }
}

impl Display for ErrorReport {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        writeln!(
            f,
            "{}: {}",
            Red.paint(format!("{}", self.line_number)),
            Red.paint(&self.message)
        )?;

        for line in self.lines.iter() {
            match line.error_span {
                None => writeln!(f, "\t{}: {}", line.line_number, &line.line)?,
                Some(span) => {
                    write!(f, "\t{}: {}", line.line_number, &line.line[..span.0])?;
                    write!(f, "{}", Red.underline().paint(&line.line[span.0..span.1]))?;
                    writeln!(f, "{}", &line.line[span.1..])?;
                }
            }
        }

        Ok(())
    }
}

impl Debug for ErrorReport {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        (self as &dyn Display).fmt(f)
    }
}

#[derive(Debug)]
pub enum SyntaxNode {
    Error(ParseError),
    Root(Box<[SyntaxNode]>),
    Rule {
        name: Token,
        production: Box<SyntaxNode>,
    },
    Plus(Box<SyntaxNode>),
    Star(Box<SyntaxNode>),
    Grouping(Box<SyntaxNode>),
    Sequence(Box<[SyntaxNode]>),
    Pipe(Box<[SyntaxNode]>),
    Identifier {
        rule_name: Token,
        member_name: Option<Token>,
    },
}

pub struct ProductionIterator<'a> {
    frontier: Vec<&'a SyntaxNode>,
}

impl SyntaxNode {
    pub fn iter(&self) -> ProductionIterator {
        ProductionIterator::from(self)
    }
}

impl<'a> From<&'a SyntaxNode> for ProductionIterator<'a> {
    fn from(root: &'a SyntaxNode) -> Self {
        ProductionIterator {
            frontier: vec![root],
        }
    }
}

impl<'a> Iterator for ProductionIterator<'a> {
    type Item = &'a SyntaxNode;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.frontier.pop()?;

        match next {
            SyntaxNode::Pipe(chain) => {
                self.frontier.extend(chain.iter().rev());
            }
            SyntaxNode::Sequence(sequence) => self.frontier.extend(sequence.iter().rev()),
            SyntaxNode::Grouping(child) | SyntaxNode::Plus(child) | SyntaxNode::Star(child) => {
                self.frontier.push(child)
            }
            SyntaxNode::Rule {
                name: _,
                production,
            } => self.frontier.push(&production),
            SyntaxNode::Error(_) => {}
            SyntaxNode::Root(items) => self.frontier.extend(items.iter().rev()),
            SyntaxNode::Identifier {
                rule_name: _,
                member_name: _,
            } => {}
        };

        Some(next)
    }
}

impl<'a> IntoIterator for &'a SyntaxNode {
    type Item = &'a SyntaxNode;
    type IntoIter = ProductionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ProductionIterator::from(self)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<SyntaxNode, Vec<ParseError>> {
    let non_trivia: Vec<Token> = tokens
        .iter()
        .filter(|t| !t.is_trivia())
        .map(|t| *t)
        .collect();

    let mut cursor = Tokens::new(non_trivia.as_slice());

    root(&mut cursor)
}

fn root(cursor: &mut Tokens) -> Result<SyntaxNode, Vec<ParseError>> {
    let mut rules: Vec<SyntaxNode> = vec![];
    let mut errors: Vec<ParseError> = vec![];

    while cursor.peek().kind != TokenKind::EndOfFile {
        match rule(cursor) {
            Ok(p) => rules.push(p),
            Err(e) => {
                errors.push(e);
                while cursor.peek().kind != TokenKind::SemiColon
                    && cursor.peek().kind != TokenKind::EndOfFile
                {
                    cursor.consume();
                }
                if cursor.peek().kind == TokenKind::SemiColon {
                    cursor.expect(TokenKind::SemiColon).unwrap();
                }
            }
        };
    }

    if errors.len() == 0 {
        Ok(SyntaxNode::Root(rules.into_boxed_slice()))
    } else {
        Err(errors)
    }
}

fn rule(cursor: &mut Tokens) -> Result<SyntaxNode, ParseError> {
    let name = cursor.expect(TokenKind::Identifier)?;
    cursor.expect(TokenKind::Arrow)?;
    let sequence = production_pipe(cursor)?;
    cursor.expect(TokenKind::SemiColon)?;

    Ok(SyntaxNode::Rule {
        name,
        production: Box::new(sequence),
    })
}

fn production_sequence(cursor: &mut Tokens) -> Result<SyntaxNode, ParseError> {
    let mut sequence: Vec<SyntaxNode> = vec![];

    loop {
        let token = cursor.peek();
        let next = match token.kind {
            TokenKind::Identifier => production_identifier(cursor)?,
            TokenKind::Plus | TokenKind::Star => production_operator(cursor, sequence.pop())?,
            TokenKind::LeftParenthesis => production_grouping(cursor)?,
            TokenKind::Quoted => {
                /* Not much to do for this right now */
                cursor.consume();
                continue;
            }
            _ => break,
        };

        sequence.push(next);
    }

    if sequence.len() != 1 {
        Ok(SyntaxNode::Sequence(sequence.into_boxed_slice()))
    } else {
        Ok(sequence.pop().unwrap())
    }
}

fn production_identifier(cursor: &mut Tokens) -> Result<SyntaxNode, ParseError> {
    let rule_name = cursor.expect(TokenKind::Identifier)?;
    let member_name = if cursor.peek().kind == TokenKind::LeftBracket {
        cursor.expect(TokenKind::LeftBracket)?;
        let name = cursor.expect(TokenKind::Identifier)?;
        cursor.expect(TokenKind::RightBracket)?;
        Some(name)
    } else {
        None
    };

    Ok(SyntaxNode::Identifier {
        rule_name,
        member_name,
    })
}

fn production_operator(
    cursor: &mut Tokens,
    lhs: Option<SyntaxNode>,
) -> Result<SyntaxNode, ParseError> {
    let lhs = lhs.ok_or(ParseError::new(
        cursor.peek(),
        ParseErrorKind::MissingSyntax {
            message: "No production sequence for operator",
        },
    ))?;
    let lhs = Box::new(lhs);
    let token = cursor.peek().kind;
    cursor.consume();

    match token {
        TokenKind::Plus => Ok(SyntaxNode::Plus(lhs)),
        TokenKind::Star => Ok(SyntaxNode::Star(lhs)),
        _ => panic!("production_sequence should've only sent production_operator a + or *"),
    }
}

fn production_grouping(cursor: &mut Tokens) -> Result<SyntaxNode, ParseError> {
    cursor.expect(TokenKind::LeftParenthesis)?;
    let sequence = production_pipe(cursor)?;
    cursor.expect(TokenKind::RightParenthesis)?;

    Ok(SyntaxNode::Grouping(Box::new(sequence)))
}

fn production_pipe(cursor: &mut Tokens) -> Result<SyntaxNode, ParseError> {
    let lhs = production_sequence(cursor)?;
    let mut chain = vec![lhs];

    while cursor.peek().kind == TokenKind::Pipe {
        cursor.consume();
        chain.push(production_sequence(cursor)?);
    }

    if chain.len() != 1 {
        Ok(SyntaxNode::Pipe(chain.into_boxed_slice()))
    } else {
        Ok(chain.pop().unwrap())
    }
}

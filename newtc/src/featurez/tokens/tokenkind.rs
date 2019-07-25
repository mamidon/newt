#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum TokenKind {
    WhiteSpace,
    CommentLine,
    CommentBlock,

    // single character tokens

    // grouping tokens
    LeftBrace,
    RightBrace,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,

    Comma,
    Dot,
    Colon,
    SemiColon,
    UnderScore,

    // math, comparison, and logic operators
    Equals,
    EqualsEquals,
    Plus,
    Minus,
    Star,
    Slash,
    Greater,
    GreaterEquals,
    Less,
    LessEquals,
    Ampersand,
    AmpersandAmpersand,
    Pipe,
    PipePipe,
    Bang,

    // literals
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    GlyphLiteral,

    Identifier,

    // keywords
    Fn,
    Return,
    If,
    Else,
    For,
    In,
    While,
    Let,
    True,
    False,

    EndOfFile,
    TombStone,
}

impl TokenKind {
	pub fn is_trivia(&self) -> bool {
		match self {
			TokenKind::TombStone
			| TokenKind::CommentLine
			| TokenKind::CommentLine
			| TokenKind::WhiteSpace
			=> true,
			_ => false
		}
	}
	
	pub fn is_binary_operator(&self) -> bool {
		match self {
			TokenKind::Plus
			| TokenKind::Minus
			| TokenKind::Star
			| TokenKind::Slash
			=> true,
			_ => false
		}
	}
	
	pub fn is_unary_operator(&self) -> bool {
		match self {
			TokenKind::Bang
			| TokenKind::Minus
			=> true,
			_ => false
		}
	}
}
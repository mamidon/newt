#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
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
	TombStone
}

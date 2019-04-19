use super::*;

use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::tokenize;
use crate::featurez::tokens::TokenKind;

#[test]
fn parse_from_tokens() {
	let text = "a+b+c";
	let tokens = tokenize(text);
	let source = StrTokenSource::new(tokens);
	let mut sink = TextTreeSink::new();
	
	sink.begin_node(SyntaxKind::BinaryExpr, 0);
		sink.attach_token(SyntaxToken::new(TokenKind::Identifier, 1));
		sink.attach_token(SyntaxToken::new(TokenKind::Plus, 1));

		sink.begin_node(SyntaxKind::BinaryExpr, 2);
			sink.attach_token(SyntaxToken::new(TokenKind::Identifier, 1));
			sink.attach_token(SyntaxToken::new(TokenKind::Plus, 1));
			sink.attach_token(SyntaxToken::new(TokenKind::Identifier, 1));
		sink.end_node(5);
	sink.end_node(5);
	
	let root= &sink.end_tree();
	let tree = SyntaxTree::new(&root, text);
	println!("{}", tree);
}

use crate::featurez::tokens::StrTokenSource;
use crate::featurez::tokens::tokenize;
use super::{TextTreeSink, TokenSource, SyntaxKind};
use super::TransparentNewType;
use crate::featurez::syntax::TreeSink;
use crate::featurez::syntax::SyntaxToken;
use crate::featurez::tokens::TokenKind;
use crate::featurez::syntax::SyntaxElement;
use crate::featurez::syntax::BinaryExprNode;
use crate::featurez::syntax::SyntaxTree;

#[test]
fn parse_from_tokens() {
	let text = "a+b+c";
	let tokens = tokenize(text);
	let source = StrTokenSource::new(text, tokens);
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

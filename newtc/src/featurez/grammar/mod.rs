use self::expr::expr;
use self::stmt::stmt;

use crate::featurez::parse::{CompletedParsing, Parser};
use crate::featurez::syntax::SyntaxKind;
use crate::featurez::TokenKind;

mod expr;
mod stmt;

pub fn root_stmt(mut p: Parser) -> CompletedParsing {
    let node = p.begin_node();

    loop {
        if p.current() == TokenKind::EndOfFile {
            break;
        }

        stmt(&mut p);
    }

    p.end_node(node, SyntaxKind::StmtListStmt);

    p.end_parsing()
}

pub fn root_expr(mut p: Parser) -> CompletedParsing {
    expr(&mut p);

    p.end_parsing()
}

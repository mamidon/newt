use self::expr::expr;
use self::stmt::stmt;

use crate::featurez::parse::{CompletedParsing, Parser};

mod expr;
mod stmt;

pub fn root_stmt(mut p: Parser) -> CompletedParsing {
    stmt(&mut p);

    p.end_parsing()
}

pub fn root_expr(mut p: Parser) -> CompletedParsing {
    expr(&mut p);

    p.end_parsing()
}

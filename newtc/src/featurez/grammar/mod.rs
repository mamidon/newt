use self::expr::expr;
use self::stmt::stmt;

use crate::featurez::parse::Parser;

mod expr;
mod stmt;

pub fn root(p: &mut Parser) {
    stmt(p);
}

pub fn root_expr(p: &mut Parser) {
    expr(p);
}
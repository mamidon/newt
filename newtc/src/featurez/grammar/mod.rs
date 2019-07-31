use self::expr::*;
use crate::featurez::parse::Parser;

mod expr;
mod stmt;

pub fn root(p: &mut Parser) {
    expr(p);
}

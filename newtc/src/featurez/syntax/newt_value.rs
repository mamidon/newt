use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};
use std::str::FromStr;

use super::NewtResult;
use super::NewtRuntimeError;
use crate::featurez::runtime::Callable;
use crate::featurez::syntax::{NewtObject, NewtString};

use crate::featurez::syntax::{
    AstNode, BinaryExprNode, ExprKind, ExprNode, GroupingExprNode, PrimitiveLiteralExprNode,
    SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTree, TextTreeSink, TokenSource,
    TreeSink, UnaryExprNode,
};
use crate::featurez::TokenKind;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum NewtValue {
    Int(i64),
    Float(f64),
    Glyph(char),
    String(NewtString),
    Bool(bool),
    Callable(Rc<dyn Callable>),
    Object(NewtObject),
    Null,
}

impl NewtValue {
    pub fn from_node(node: &SyntaxNode) -> Option<NewtValue> {
        ExprNode::cast(node)
            .and_then(|n| Some(n.kind()))
            .and_then(|k| {
                if let ExprKind::PrimitiveLiteralExpr(l) = k {
                    Some(l)
                } else {
                    None
                }
            })
            .and_then(|e| Some(NewtValue::from_primitive_literal_node(e)))
    }

    pub fn from_primitive_literal_node(node: &PrimitiveLiteralExprNode) -> NewtValue {
        let literal = node.literal();
        let lexeme = literal.lexeme();

        match literal.token_kind() {
            TokenKind::IntegerLiteral => {
                NewtValue::Int(lexeme.parse().expect("unparsable literal token"))
            }
            TokenKind::FloatLiteral => {
                NewtValue::Float(lexeme.parse().expect("unparsable literal token"))
            }
            TokenKind::StringLiteral => {
                NewtValue::String(NewtString::new(&lexeme[1..lexeme.len() - 1]))
            }
            TokenKind::GlyphLiteral => {
                NewtValue::Glyph(lexeme[1..2].parse().expect("unparsable literal token"))
            }
            TokenKind::True => NewtValue::Bool(true),
            TokenKind::False => NewtValue::Bool(false),
            _ => panic!("Literal node has non-literal token"),
        }
    }

    pub fn as_truthy(&self) -> Option<bool> {
        match self {
            NewtValue::Bool(b) => Some(*b),
            NewtValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }
}

impl Add for NewtValue {
    type Output = NewtResult;

    fn add(self, rhs: Self) -> <Self as Add<Self>>::Output {
        match (self, rhs) {
            (NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l + r)),
            (NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l + r)),
            _ => Err(NewtRuntimeError::TypeError),
        }
    }
}

impl Sub for NewtValue {
    type Output = NewtResult;

    fn sub(self, rhs: Self) -> <Self as Add<Self>>::Output {
        match (self, rhs) {
            (NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l - r)),
            (NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l - r)),
            _ => Err(NewtRuntimeError::TypeError),
        }
    }
}

impl Mul for NewtValue {
    type Output = NewtResult;

    fn mul(self, rhs: Self) -> <Self as Add<Self>>::Output {
        match (self, rhs) {
            (NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l * r)),
            (NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l - r)),
            _ => Err(NewtRuntimeError::TypeError),
        }
    }
}

impl Div for NewtValue {
    type Output = NewtResult;

    fn div(self, rhs: Self) -> <Self as Add<Self>>::Output {
        match (self, rhs) {
            (NewtValue::Int(l), NewtValue::Int(r)) => Ok(NewtValue::Int(l / r)),
            (NewtValue::Float(l), NewtValue::Float(r)) => Ok(NewtValue::Float(l / r)),
            _ => Err(NewtRuntimeError::TypeError),
        }
    }
}

impl Not for NewtValue {
    type Output = NewtResult;

    fn not(self) -> <Self as Not>::Output {
        match self {
            _ => Err(NewtRuntimeError::TypeError),
        }
    }
}

impl Neg for NewtValue {
    type Output = NewtResult;

    fn neg(self) -> <Self as Neg>::Output {
        match self {
            NewtValue::Int(l) => Ok(NewtValue::Int(-l)),
            NewtValue::Float(l) => Ok(NewtValue::Float(-l)),
            _ => Err(NewtRuntimeError::TypeError),
        }
    }
}

impl PartialOrd for NewtValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (NewtValue::Int(a), NewtValue::Int(b)) => {
                if a < b {
                    Some(Ordering::Less)
                } else if a == b {
                    Some(Ordering::Equal)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (NewtValue::Float(a), NewtValue::Float(b)) => {
                if a < b {
                    Some(Ordering::Less)
                } else if a == b {
                    Some(Ordering::Equal)
                } else {
                    Some(Ordering::Greater)
                }
            }
            _ => None,
        }
    }
}

impl PartialEq for NewtValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NewtValue::Int(a), NewtValue::Int(b)) => a == b,
            (NewtValue::Float(a), NewtValue::Float(b)) => a == b,
            (NewtValue::Bool(a), NewtValue::Bool(b)) => a == b,
            (NewtValue::String(a), NewtValue::String(b)) => a == b,
            (NewtValue::Glyph(a), NewtValue::Glyph(b)) => a == b,
            (NewtValue::Null, NewtValue::Null) => true,
            _ => false,
        }
    }
}

impl From<&str> for NewtValue {
    fn from(s: &str) -> Self {
        NewtValue::String(NewtString::new(s))
    }
}

impl From<i64> for NewtValue {
    fn from(i: i64) -> Self {
        NewtValue::Int(i)
    }
}

impl From<i32> for NewtValue {
    fn from(i: i32) -> Self {
        NewtValue::Int(i.into())
    }
}

impl From<u32> for NewtValue {
    fn from(i: u32) -> Self {
        NewtValue::Int(i.into())
    }
}

impl From<char> for NewtValue {
    fn from(c: char) -> Self {
        NewtValue::Glyph(c)
    }
}

impl From<bool> for NewtValue {
    fn from(b: bool) -> Self {
        NewtValue::Bool(b)
    }
}

impl From<f64> for NewtValue {
    fn from(f: f64) -> Self {
        NewtValue::Float(f)
    }
}

impl From<f32> for NewtValue {
    fn from(f: f32) -> Self {
        NewtValue::Float(f.into())
    }
}

impl From<NewtObject> for NewtValue {
    fn from(o: NewtObject) -> Self {
        NewtValue::Object(o)
    }
}

impl From<NewtString> for NewtValue {
    fn from(s: NewtString) -> Self {
        NewtValue::String(s)
    }
}

impl<T> From<Option<T>> for NewtValue
where
    T: Into<NewtValue>,
{
    fn from(maybe: Option<T>) -> Self {
        match maybe {
            Some(value) => value.into(),
            None => NewtValue::Null,
        }
    }
}

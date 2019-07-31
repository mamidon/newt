mod expr_visitor;
mod newt_runtime_error;
mod newt_value;

pub use self::expr_visitor::{ExprVisitor, ExprVirtualMachine};
pub use self::newt_runtime_error::NewtRuntimeError;
pub use self::newt_value::NewtValue;

type NewtResult = Result<NewtValue, NewtRuntimeError>;



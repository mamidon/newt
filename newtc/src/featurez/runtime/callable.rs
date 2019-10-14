use crate::featurez::syntax::{FunctionDeclarationStmtNode, NewtRuntimeError, NewtValue, FunctionCallExprNode};
use crate::featurez::VirtualMachineState;
use std::fmt::{Debug, Error, Formatter};
use std::convert::TryFrom;
use std::collections::HashMap;

pub trait Callable {
    fn symbol(&self) -> &str;
    fn arity(&self) -> usize;
	fn call(
		&self,
		vm: &mut VirtualMachineState,
		arguments: &[NewtValue]
	) -> Result<NewtValue, NewtRuntimeError>;
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<fn {}:{}>", self.arity(), self.symbol())
    }
}


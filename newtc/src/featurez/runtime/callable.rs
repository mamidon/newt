use crate::featurez::syntax::{FunctionDeclarationStmtNode, NewtRuntimeError, NewtValue};
use crate::featurez::VirtualMachineState;
use std::fmt::{Debug, Error, Formatter};
use std::convert::TryFrom;
use std::collections::HashMap;

pub trait Callable {
    fn symbol(&self) -> &str;
    fn arity(&self) -> usize;
	fn call(
		&mut self,
		vm: &mut VirtualMachineState
	) -> Result<Option<NewtValue>, NewtRuntimeError>;
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<fn {}:{}>", self.arity(), self.symbol())
    }
}

impl Callable for FunctionDeclarationStmtNode {
    fn symbol(&self) -> &str {
        self.identifier().lexeme()
    }

    fn arity(&self) -> usize {
        self.arguments().count()
    }

    fn call(
	    &mut self,
	    vm: &mut VirtualMachineState,
    ) -> Result<Option<NewtValue>, NewtRuntimeError> {
        unimplemented!()
    }
}

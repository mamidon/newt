use crate::featurez::syntax::{FunctionDeclarationStmtNode, NewtRuntimeError, NewtValue};
use crate::featurez::VirtualMachine;
use std::fmt::{Debug, Error, Formatter};

pub trait Callable {
    fn symbol(&self) -> &str;
    fn arity(&self) -> usize;
    fn call(
        &mut self,
        vm: &mut VirtualMachine,
        arguments: &[NewtValue],
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
        vm: &mut VirtualMachine,
        arguments: &[NewtValue],
    ) -> Result<Option<NewtValue>, NewtRuntimeError> {
        unimplemented!()
    }
}
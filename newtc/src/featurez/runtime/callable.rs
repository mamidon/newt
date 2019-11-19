use crate::featurez::syntax::{FunctionDeclarationStmtNode, NewtRuntimeError, NewtValue, FunctionCallExprNode, StmtVisitor};
use crate::featurez::VirtualMachineState;
use std::fmt::{Debug, Error, Formatter};
use std::convert::TryFrom;
use std::collections::HashMap;
use crate::featurez::runtime::scope::{ScopeNode, Environment};
use crate::featurez::runtime::VirtualMachineInterpretingSession;

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

pub struct NewtCallable {
	definition: FunctionDeclarationStmtNode,
	closure: Environment
}

impl NewtCallable {
	pub fn new(node: &FunctionDeclarationStmtNode, closure: &Environment) -> NewtCallable {
		NewtCallable {
			definition: node.clone(),
			closure: closure.clone()
		}
	}
}

impl Callable for NewtCallable {
	fn symbol(&self) -> &str {
		self.definition.identifier().lexeme()
	}

	fn arity(&self) -> usize {
		self.definition.arguments().count()
	}

	fn call(&self, vm: &mut VirtualMachineState, arguments: &[NewtValue]) -> Result<NewtValue, NewtRuntimeError> {
		let mut environment = self.closure.clone();
		environment.push_scope();

		for parameter in self.definition.arguments().enumerate() {
			environment.bind(parameter.1.lexeme(), arguments[parameter.0].clone());
		}

		let mut next_vm = VirtualMachineState::new_with_scope(&environment);
		let result = next_vm.visit_stmt_list_stmt(self.definition.stmts());
		environment.pop_scope();

		match result {
			Ok(value) => Ok(NewtValue::Null),
			Err(NewtRuntimeError::ReturnedValue(value)) => Ok(value),
			Err(error) => Err(error)
		}
	}
}
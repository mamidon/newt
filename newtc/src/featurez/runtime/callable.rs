use crate::featurez::syntax::{FunctionDeclarationStmtNode, NewtRuntimeError, NewtValue, FunctionCallExprNode, StmtVisitor};
use crate::featurez::VirtualMachineState;
use std::fmt::{Debug, Error, Formatter};
use std::convert::TryFrom;
use std::collections::HashMap;
use crate::featurez::runtime::scope::LexicalScope;
use crate::featurez::runtime::VirtualMachineInterpretingSession;

pub trait Callable {
    fn symbol(&self) -> &str;
    fn arity(&self) -> usize;
	fn call(
		&self,
		vm: &mut VirtualMachineInterpretingSession,
		arguments: &[NewtValue]
	) -> Result<NewtValue, NewtRuntimeError>;
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<fn {}:{}>", self.arity(), self.symbol())
    }
}

struct NewtCallable {
	definition: FunctionDeclarationStmtNode,
	closure: LexicalScope
}

impl NewtCallable {
	pub fn new(node: FunctionDeclarationStmtNode, closure: LexicalScope) -> NewtCallable {
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

	fn call(&self, vm: &mut VirtualMachineInterpretingSession, arguments: &[NewtValue]) -> Result<NewtValue, NewtRuntimeError> {
		vm.visit_stmt_list_stmt(self.definition.stmts());

		Ok(NewtValue::Null)
	}
}
use crate::featurez::runtime::scope::{Environment, ScopeNode};
use crate::featurez::syntax::{
    FunctionCallExprNode, FunctionDeclarationStmtNode, NewtRuntimeError, NewtValue, StmtVisitor,
};
use crate::featurez::VirtualMachine;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Error, Formatter};

pub trait Callable {
    fn symbol(&self) -> &str;
    fn arity(&self) -> usize;
    fn call(
        &self,
        vm: &mut VirtualMachine,
        arguments: &[NewtValue],
    ) -> Result<NewtValue, NewtRuntimeError>;
}

impl Debug for dyn Callable {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<fn {}:{}>", self.arity(), self.symbol())
    }
}

pub struct NewtCallable {
    definition: FunctionDeclarationStmtNode,
    closure: Environment,
}

impl NewtCallable {
    pub fn new(node: &FunctionDeclarationStmtNode, closure: &Environment) -> NewtCallable {
        NewtCallable {
            definition: node.clone(),
            closure: closure.clone(),
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

    fn call(
        &self,
        vm: &mut VirtualMachine,
        arguments: &[NewtValue],
    ) -> Result<NewtValue, NewtRuntimeError> {
        let mut environment = self.closure.clone();
        environment.push_scope();

        for parameter in self.definition.arguments().enumerate() {
            environment.bind(parameter.1.lexeme(), arguments[parameter.0].clone());
        }

        let mut next_vm = VirtualMachine::new_with_scope(&environment);
        let result = next_vm.visit_stmt_list_stmt(self.definition.stmts());
        environment.pop_scope();

        match result {
            Ok(value) => Ok(NewtValue::Null),
            Err(NewtRuntimeError::ReturnedValue(value)) => Ok(value),
            Err(error) => Err(error),
        }
    }
}

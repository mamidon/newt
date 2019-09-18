use std::collections::HashMap;
use crate::featurez::syntax::{NewtValue, NewtRuntimeError};
use crate::featurez::syntax::NewtResult;
use std::process::id;

#[derive(Debug)]
pub struct Scope {
	stack: Vec<Box<HashMap<String, NewtValue>>>,
	values: HashMap<String, NewtValue>
}

impl Scope {
	pub fn new() -> Scope {
		Scope {
			stack: vec![],
			values: HashMap::new()
		}
	}
	
	pub fn push_scope(&mut self) {
		use std::mem::replace;
		
		let previous = replace(&mut self.values, HashMap::new());
		self.stack.push(Box::new(previous));
	}
	
	pub fn pop_scope(&mut self) {
		match self.stack.pop() {
			Some(scope) => self.values = *scope,
			None => self.values = HashMap::new()
		}
	}
	
	pub fn declare(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
		if self.values.contains_key(identifier) {
			Err(NewtRuntimeError::DuplicateDeclaration)
		} else {
			self.values.insert(identifier.to_owned(), value);
			Ok(())
		}
	}

	pub fn assign(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
		if self.values.contains_key(identifier) {
			self.values.insert(identifier.to_owned(), value);

			return Ok(());
		}

		for scope in self.stack.iter_mut().rev() {
			if scope.contains_key(identifier) {
				scope.insert(identifier.to_owned(), value);
				return Ok(());
			}
		}

		return Err(NewtRuntimeError::UndefinedVariable);
	}
	
	pub fn resolve(&self, identifier: &str) -> Option<&NewtValue> {
		if self.values.contains_key(identifier) {
			return self.values.get(identifier);
		}

		for scope in self.stack.iter().rev() {
			if scope.contains_key(identifier) {
				return scope.get(identifier);
			}
		}
				
		return None;
	}
}

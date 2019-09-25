use std::collections::HashMap;
use crate::featurez::syntax::{NewtValue, NewtRuntimeError};
use crate::featurez::syntax::NewtResult;
use std::rc::Rc;
use std::cell::RefCell;

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
	
	pub fn declare(&mut self, identifier: &str, value: NewtValue) -> Option<NewtRuntimeError> {
		if self.values.contains_key(identifier) {
			Some(NewtRuntimeError::DuplicateDeclaration)
		} else {
			self.values.insert(identifier.to_owned(), value);
			None
		}
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

	pub fn resolve_mut(&mut self, identifier: &str) -> Option<&mut NewtValue> {
		if self.values.contains_key(identifier) {
			return self.values.get_mut(identifier);
		}

		for scope in self.stack.iter_mut().rev() {
			if scope.contains_key(identifier) {
				return scope.get_mut(identifier);
			}
		}

		return None;
	}
}


type ScopeMap = Rc<RefCell<HashMap<String, NewtValue>>>;

struct OwningScope
{
	parent: Option<ScopeMap>,
	scope: ScopeMap
}

#[derive(Debug)]
pub struct LexicalScope {
	stack: OwningScope,
}


pub struct ClosedScope {
	scope: OwningScope, // I've got to Rc<> something.. perhaps the scope not the hashmap?
	sequence_number: usize
}
pub struct ClosedValue;

impl LexicalScope {
	fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> { unimplemented!() }
	fn push(&mut self) {}
	fn pop(&mut self) {}
}


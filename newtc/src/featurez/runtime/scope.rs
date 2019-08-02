use std::collections::HashMap;
use crate::featurez::syntax::NewtValue;

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
	
	pub fn bind(&mut self, identifier: &str, value: NewtValue) {
		self.values.insert(identifier.to_string(), value);
	}
	
	pub fn resolve(&self, identifier: &str) -> Option<&NewtValue> {
		if self.values.contains_key(identifier) {
			return Some(&self.values[identifier]);
		}
		
		for scope in self.stack.iter().rev() {
			if scope.contains_key(identifier) {
				return Some(&scope[identifier]);
			}
		}
				
		return None;
	}
}

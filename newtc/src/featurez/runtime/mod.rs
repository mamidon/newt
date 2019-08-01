mod scope {
	use std::collections::HashMap;
	use crate::featurez::syntax::NewtValue;
	
	pub struct Scope<'a> {
		stack: Vec<Box<HashMap<&'a str, NewtValue>>>,
		values: HashMap<&'a str, NewtValue>
	}
	
	impl<'a> Scope<'a> {
		pub fn new() -> Scope<'a> {
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
		
		pub fn bind(&mut self, identifier: &'a str, value: NewtValue) {
			self.values.insert(identifier, value);
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
}
mod expr_virtual_machine;

pub use self::expr_virtual_machine::ExprVirtualMachine;




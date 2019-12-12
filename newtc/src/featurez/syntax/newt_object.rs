use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::featurez::syntax::NewtValue;
use std::fmt::{Display, Formatter, Error, Debug};
use std::collections::hash_map::Keys;

type InternalObject = Rc<RefCell<HashMap<String, NewtValue>>>;

#[derive(Clone)]
pub struct NewtObject(InternalObject);

impl NewtObject {
	pub fn new() -> NewtObject {
		NewtObject(Rc::new(RefCell::new(HashMap::new())))
	}

	pub fn get(&self, name: &str) -> Option<NewtValue> {
		self.0.borrow().get(name).map(|v| v.clone())
	}

	pub fn set(&mut self, name: &str, value: &NewtValue) -> &mut Self {
		self.0.borrow_mut().insert(name.to_string(), value.clone());

		self
	}

	pub fn keys(&self) -> Vec<String> {
		self.0.borrow()
			.keys()
			.map(|k| k.clone())
			.collect()
	}
}

impl Display for NewtObject {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{:#?}", self.0.borrow())
	}
}

impl Debug for NewtObject {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{:#?}", self.0.borrow())
	}
}
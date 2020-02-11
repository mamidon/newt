use std::fmt::{Debug, Display, Error, Formatter};
use std::rc::Rc;

#[derive(Clone, PartialOrd, PartialEq)]
pub struct NewtString(Rc<String>);

impl NewtString {
    pub fn new(s: &str) -> NewtString {
        NewtString(Rc::new(s.to_string()))
    }
}

impl Debug for NewtString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for NewtString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

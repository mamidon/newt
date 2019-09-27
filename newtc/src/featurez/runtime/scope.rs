use crate::featurez::syntax::NewtResult;
use crate::featurez::syntax::{NewtRuntimeError, NewtValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Scope {
    stack: Vec<Box<HashMap<String, NewtValue>>>,
    values: HashMap<String, NewtValue>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            stack: vec![],
            values: HashMap::new(),
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
            None => self.values = HashMap::new(),
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


type ScopeMap = HashMap<String, StoredValue>;

type ScopeMapLink = Rc<RefCell<ScopeMap>>;
type ScopeNodeLink = Rc<RefCell<ScopeNode>>;

struct StoredValue {
    sequence_number: usize,
    value: NewtValue
}

#[derive(Clone)]
struct ScopeNode {
    next: Option<ScopeNodeLink>,
    scope: ScopeMapLink
}

#[derive(Clone)]
struct LexicalScope {
    top: ScopeNodeLink,
    sequence_number: usize
}

impl ScopeNode {
    fn new() -> ScopeNode {
        ScopeNode {
            scope: Rc::new(RefCell::new(HashMap::new())),
            next: None
        }
    }
}

impl ScopeNode {
    fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        let mut hash_map = self.scope.borrow_mut();

        if hash_map.contains_key(identifier) {
            return Err(NewtRuntimeError::DuplicateDeclaration);
        }

        let stored_value = StoredValue::new(hash_map.len(), value);
        hash_map.insert(identifier.to_string(), stored_value);

        Ok(())
    }

    fn resolve(&self, identifier: &str, sequence_number: usize) -> Result<NewtValue, NewtRuntimeError> {
        let scope = self.scope.borrow();
        if let Some(stored_value) = scope.get(identifier) {
            if stored_value.sequence_number < sequence_number {
                return Ok(stored_value.value.clone());
            }
        }

        match &self.next {
            Some(next) => {
                next.borrow().resolve(identifier, sequence_number)
            },
            None => Err(NewtRuntimeError::UndefinedVariable)
        }
    }
}

impl LexicalScope {
    fn new() -> LexicalScope {
        LexicalScope {
            top: Rc::new(RefCell::new(ScopeNode::new())),
            sequence_number: 0
        }
    }

    fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        self.top.borrow_mut().bind(identifier, value)?;
        self.sequence_number += 1;
        Ok(())
    }

    fn resolve(&self, identifier: &str) -> Result<NewtValue, NewtRuntimeError> {
        self.top.borrow().resolve(identifier, self.sequence_number)
    }

    fn push(&mut self) {
        let mut next_top = Rc::new(RefCell::new(ScopeNode::new()));
        (*next_top).borrow_mut().next = Some(self.top.clone());
        self.top = next_top;
    }

    fn pop(&mut self) {
        let next = self.top.borrow_mut().next.take().unwrap();
        self.top = next.clone();
    }
}

impl StoredValue {
    fn new(sequence_number: usize, value: NewtValue) -> StoredValue {
        StoredValue {
            sequence_number,
            value
        }
    }
}

mod tests {
    use crate::featurez::runtime::scope::LexicalScope;
    use crate::featurez::syntax::{NewtValue, NewtRuntimeError};

    #[test]
    pub fn lexical_scope_can_resolve_immediately_after_binding() {
        let mut scope = LexicalScope::new();

        scope.bind("foo", NewtValue::Int(42)).unwrap();
        scope.bind("bar", NewtValue::Int(32)).unwrap();

        assert_eq!(Ok(NewtValue::Int(42)), scope.resolve("foo"));
        assert_eq!(Ok(NewtValue::Int(32)), scope.resolve("bar"));
    }

    #[test]
    pub fn lexical_scope_returns_undefined_variable_error_when_resolving_fails() {
        let mut scope = LexicalScope::new();

        assert_eq!(Err(NewtRuntimeError::UndefinedVariable), scope.resolve("zoo"));
    }

    #[test]
    pub fn lexical_scope_can_resolv_top_scope_when_scopes_are_nested() {
        let mut scope = LexicalScope::new();

        scope.bind("foo", NewtValue::Int(42)).unwrap();
        scope.push();
        scope.bind("bar", NewtValue::Int(32)).unwrap();
        scope.pop();
        scope.bind("zoo", NewtValue::Int(22)).unwrap();

        assert_eq!(Ok(NewtValue::Int(42)), scope.resolve("foo"));
        assert_eq!(Ok(NewtValue::Int(32)), scope.resolve("bar"));
        assert_eq!(Ok(NewtValue::Int(22)), scope.resolve("zoo"));
    }

    #[test]
    pub fn closed_lexical_scope_cannot_resolve_younger_variables_in_parent_scope() {
        let mut scope = LexicalScope::new();

        scope.bind("foo", NewtValue::Int(42)).unwrap();
        scope.push();
        let mut closure = scope.clone();
        closure.bind("bar", NewtValue::Int(32)).unwrap();
        scope.pop();
        scope.bind("zoo", NewtValue::Int(22)).unwrap();

        assert_eq!(Ok(NewtValue::Int(42)), closure.resolve("foo"));
        assert_eq!(Ok(NewtValue::Int(32)), closure.resolve("bar"));

        assert_eq!(Err(NewtRuntimeError::UndefinedVariable), closure.resolve("zoo"));
        assert_eq!(Ok(NewtValue::Int(22)), scope.resolve("zoo"));
    }
}

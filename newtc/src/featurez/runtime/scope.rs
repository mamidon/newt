use crate::featurez::syntax::NewtResult;
use crate::featurez::syntax::{NewtRuntimeError, NewtValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::borrow::{Borrow, BorrowMut};
use std::process::id;

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


type ScopeMap = Rc<RefCell<HashMap<String, StoredValue>>>;

struct StoredValue {
    sequence_number: usize,
    value: NewtValue
}

#[derive(Clone)]
struct ScopeNode {
    next: Option<Rc<ScopeNode>>,
    scope: ScopeMap
}

struct ScopeStack {
    scopes: Vec<ScopeNode>
}

pub struct LexicalScope {
    scope: ScopeStack,
}

pub struct ClosedScope {
    scope: ScopeNode,
    sequence_number: usize,
}
pub struct ClosedValue;

impl LexicalScope {
    fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        self.scope.bind(identifier, value)
    }

    fn push(&mut self) {
        self.scope.push()
    }

    fn pop(&mut self) {
        self.scope.pop()
    }

    fn close(&self) -> ClosedScope {
        ClosedScope::from(&self.scope)
    }
}

impl From<&ScopeStack> for ClosedScope {
    fn from(scope_stack: &ScopeStack) -> Self {
        ClosedScope {
            scope: scope_stack.peek().clone(),
            sequence_number: (*scope_stack.peek().scope).borrow().len()
        }
    }
}

impl ScopeNode {
    fn new() -> ScopeNode {
        ScopeNode {
            scope: Rc::new(RefCell::new(HashMap::new())),
            next: None
        }
    }
}

impl ScopeStack {
    fn new() -> ScopeStack {
        ScopeStack {
            scopes: vec![ScopeNode::new()]
        }
    }

    fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        let mut hash_map = (*self.peek().scope).borrow_mut();

        if hash_map.contains_key(identifier) {
            return Err(NewtRuntimeError::DuplicateDeclaration);
        }

        let stored_value = StoredValue::new(hash_map.len(), value);
        hash_map.insert(identifier.to_string(), stored_value);

        Ok(())
    }

    fn push(&mut self) {
        self.scopes.push(ScopeNode::new())
    }

    fn pop(&mut self) {
        match self.scopes.pop() {
            Some(_) => {},
            None => panic!("Ran out of scopes to pop!")
        }
    }

    fn peek(&self) -> &ScopeNode {
        &self.scopes[self.scopes.len() - 1]
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

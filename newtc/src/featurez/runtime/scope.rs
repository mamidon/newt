use crate::featurez::syntax::{NewtResult};
use crate::featurez::syntax::{NewtRuntimeError, NewtValue};
use crate::featurez::syntax::{SyntaxElement};
use crate::featurez::syntax::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::hash::{Hash, Hasher};
use crate::featurez::newtypes::TransparentNewType;

type ScopeMap = HashMap<String, StoredValue>;

type ScopeMapLink = Rc<RefCell<ScopeMap>>;
type ScopeNodeLink = Rc<RefCell<ScopeNode>>;

#[derive(Debug)]
struct StoredValue {
    value: NewtValue
}

#[derive(Clone, Debug)]
struct ScopeNode {
    next: Option<ScopeNodeLink>,
    scope: ScopeMapLink
}

#[derive(Clone, Debug)]
pub struct LexicalScope {
    top: ScopeNodeLink,
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

        let stored_value = StoredValue::new(value);
        hash_map.insert(identifier.to_string(), stored_value);

        Ok(())
    }

    fn assign(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        let mut scope = self.scope.borrow_mut();
        if let Some(stored_value) = scope.get_mut(identifier) {
            *stored_value = StoredValue::new(value);
            return Ok(());
        }

        match &self.next {
            Some(next) => {
                next.borrow_mut().assign(identifier, value)
            },
            None => Err(NewtRuntimeError::UndefinedVariable)
        }
    }

    fn resolve(&self, identifier: &str) -> Result<NewtValue, NewtRuntimeError> {
        let scope = self.scope.borrow();
        if let Some(stored_value) = scope.get(identifier) {
            return Ok(stored_value.value.clone());
        }

        match &self.next {
            Some(next) => {
                next.borrow().resolve(identifier)
            },
            None => Err(NewtRuntimeError::UndefinedVariable)
        }
    }
}

impl LexicalScope {
    pub fn new() -> LexicalScope {
        LexicalScope {
            top: Rc::new(RefCell::new(ScopeNode::new()))
        }
    }

    pub fn new_with_closure(closure: &LexicalScope) -> LexicalScope {
        LexicalScope {
            top: closure.top.clone()
        }
    }

    pub fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        self.top.borrow_mut().bind(identifier, value)?;
        Ok(())
    }

    pub fn resolve(&self, identifier: &str) -> Result<NewtValue, NewtRuntimeError> {
        unimplemented!()
    }

    pub fn assign(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        self.top.borrow_mut().assign(identifier, value)
    }

    pub fn push(&mut self) {
        let mut next_top = Rc::new(RefCell::new(ScopeNode::new()));
        (*next_top).borrow_mut().next = Some(self.top.clone());
        self.top = next_top;
    }

    pub fn pop(&mut self) {
        let next = self.top.borrow().next.clone().unwrap();
        self.top = next;
    }
}

impl StoredValue {
    fn new(value: NewtValue) -> StoredValue {
        StoredValue {
            value
        }
    }
}

#[derive(Debug)]
enum DeclarationProgress {
    Undeclared,
    BeingDeclared,
    Declared
}

#[derive(Debug)]
pub struct RefEquality<T>(*const T);

impl<T> Hash for RefEquality<T>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const T).hash(state)
    }
}

impl<T> PartialEq<RefEquality<T>> for RefEquality<T>
{
    fn eq(&self, other: &RefEquality<T>) -> bool {
        (self.0 as *const T) == (other.0 as *const T)
    }
}

impl<T> From<&T> for RefEquality<T>
{
    fn from(item: &T) -> Self {
        RefEquality(item as *const T)
    }
}

impl<T> Eq for RefEquality<T> {}

mod lexical_scope_analyzer_tests {
    use crate::featurez::runtime::scope::{LexicalScope, RefEquality};
    use crate::featurez::syntax::{NewtValue, NewtRuntimeError, SyntaxToken, SyntaxTree, StmtNode, AstNode, SyntaxElement, SyntaxNode, WhileStmtNode, SyntaxKind, NewtStaticError, VariableExprNode, VariableAssignmentStmtNode};
    use crate::featurez::grammar::root_stmt;
    use crate::featurez::{InterpretingSession, InterpretingSessionKind};
    use crate::featurez::newtypes::TransparentNewType;
    use std::collections::HashMap;
    use std::borrow::Borrow;

    fn source_to_tree(source: &str) -> InterpretingSession {
        InterpretingSession::new(InterpretingSessionKind::Stmt, source)
    }

    fn tree_to_variable_references<'a>(tree: &'a SyntaxTree, identifier: &str) -> Vec<&'a SyntaxNode> {
        tree.iter()
            .filter_map(|e|
                e.as_node().filter(|n| n.kind() == SyntaxKind::VariableExpr))
            .map(|n| VariableExprNode::from_inner(n))
            .filter(|v| v.identifier().lexeme() == identifier)
            .map(|n| n.to_inner())
            .collect()
    }

    fn tree_to_variable_assignments<'a>(tree: &'a SyntaxTree, identifier: &str) -> Vec<&'a SyntaxNode> {
        tree.iter()
            .filter_map(|e|
                e.as_node().filter(|n| n.kind() == SyntaxKind::VariableAssignmentStmt))
            .map(|n| VariableAssignmentStmtNode::from_inner(n))
            .filter(|v| v.identifier().lexeme() == identifier)
            .map(|n| n.to_inner())
            .collect()
    }

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
    pub fn lexical_scope_can_resolve_top_scope_when_scopes_are_nested() {
        let mut scope = LexicalScope::new();

        scope.bind("foo", NewtValue::Int(42)).unwrap();
        scope.push();
        scope.bind("bar", NewtValue::Int(32)).unwrap();
        let top_scope = scope.clone();

        scope.pop();
        scope.bind("zoo", NewtValue::Int(22)).unwrap();

        assert_eq!(Ok(NewtValue::Int(42)), scope.resolve("foo"));
        assert_eq!(Err(NewtRuntimeError::UndefinedVariable), scope.resolve("bar"));
        assert_eq!(Ok(NewtValue::Int(32)), top_scope.resolve("bar"));
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

        // For now this is allowed, we'll need static analysis to ensure that no one tries
        // to access a variable 'prior' to declaring it.  Today, saving a function variable
        // and calling it later could violate that.
        assert_eq!(Ok(NewtValue::Int(22)), closure.resolve("zoo"));
    }
}

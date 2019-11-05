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

#[derive(Clone, Debug)]
struct ScopeNodeLink {
    next: Rc<RefCell<ScopeNode>>,
    sequence_number: usize
}

#[derive(Debug)]
struct StoredValue {
    value: NewtValue,
	sequence_number: usize
}

#[derive(Clone, Debug)]
pub struct ScopeNode {
    link: Option<ScopeNodeLink>,
    scope: ScopeMapLink
}

impl ScopeNode {
    pub fn new() -> ScopeNode {
        ScopeNode {
            scope: Rc::new(RefCell::new(HashMap::new())),
            link: None
        }
    }

    pub fn new_with_scope(parent: &ScopeNode) -> ScopeNode {
        ScopeNode {
            scope: Rc::new(RefCell::new(HashMap::new())),
            link: Some(ScopeNodeLink {
                next: Rc::new(RefCell::new(parent.clone())),
                sequence_number: parent.scope.borrow().len()
            })
        }
    }
}

impl ScopeNode {
    pub fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        let mut hash_map = self.scope.borrow_mut();

        if hash_map.contains_key(identifier) {
            return Err(NewtRuntimeError::DuplicateDeclaration);
        }

        let stored_value = StoredValue::new(value, self.scope.borrow().len());
        hash_map.insert(identifier.to_string(), stored_value);

        Ok(())
    }

    pub fn assign(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        let mut scope = self.scope.borrow_mut();
        if let Some(stored_value) = scope.get_mut(identifier) {
            *stored_value = StoredValue::new(value, stored_value.sequence_number);
            return Ok(());
        }

        match &self.link {
            Some(link) => {
                link.next.borrow_mut().assign(identifier, value)
            },
            None => Err(NewtRuntimeError::UndefinedVariable)
        }
    }

    pub fn resolve(&self, identifier: &str) -> Result<NewtValue, NewtRuntimeError> {
        let scope = self.scope.borrow();
        if let Some(stored_value) = scope.get(identifier) {
            return Ok(stored_value.value.clone());
        }

        match &self.link {
            Some(link) => {
                link.next.borrow().filtered_resolve(identifier, link.sequence_number)
            },
            None => Err(NewtRuntimeError::UndefinedVariable)
        }
    }

    fn filtered_resolve(&self, identifier: &str, sequence_number: usize) -> Result<NewtValue, NewtRuntimeError> {
        let scope = self.scope.borrow();
        if let Some(stored_value) = scope.get(identifier) {
            if stored_value.sequence_number < sequence_number {
                return Ok(stored_value.value.clone());
            }
        }

        match &self.link {
            Some(link) => {
                link.next.borrow().resolve(identifier)
            },
            None => Err(NewtRuntimeError::UndefinedVariable)
        }
    }
}

impl StoredValue {
    fn new(value: NewtValue, sequence_number: usize) -> StoredValue {
        StoredValue {
            value,
            sequence_number
        }
    }
}

mod lexical_scope_analyzer_tests {
    use crate::featurez::syntax::{NewtValue, NewtRuntimeError, SyntaxToken, SyntaxTree, StmtNode, AstNode, SyntaxElement, SyntaxNode, WhileStmtNode, SyntaxKind, NewtStaticError, VariableExprNode, VariableAssignmentStmtNode};
    use crate::featurez::grammar::root_stmt;
    use crate::featurez::{InterpretingSession, InterpretingSessionKind};
    use crate::featurez::newtypes::TransparentNewType;
    use std::collections::HashMap;
    use std::borrow::Borrow;
    use crate::featurez::runtime::scope::ScopeNode;

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
        let mut scope = ScopeNode::new();

        scope.bind("foo", NewtValue::Int(42)).unwrap();
        scope.bind("bar", NewtValue::Int(32)).unwrap();

        assert_eq!(Ok(NewtValue::Int(42)), scope.resolve("foo"));
        assert_eq!(Ok(NewtValue::Int(32)), scope.resolve("bar"));
    }

    #[test]
    pub fn lexical_scope_returns_undefined_variable_error_when_resolving_fails() {
        let mut scope = ScopeNode::new();

        assert_eq!(Err(NewtRuntimeError::UndefinedVariable), scope.resolve("zoo"));
    }

    #[test]
    pub fn lexical_scope_can_resolve_top_scope_when_scopes_are_nested() {
        let mut scope = ScopeNode::new();

        scope.bind("foo", NewtValue::Int(42)).unwrap();
        scope = ScopeNode::new_with_scope(&scope);
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
        let mut scope = ScopeNode::new();

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

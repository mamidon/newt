use crate::featurez::syntax::{NewtResult, ExprVisitor};
use crate::featurez::syntax::{NewtRuntimeError, NewtValue};
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

    pub fn bind(&mut self, identifier: &str, value: NewtValue) -> Result<(), NewtRuntimeError> {
        self.top.borrow_mut().bind(identifier, value)?;
        Ok(())
    }

    pub fn resolve(&self, identifier: &str) -> Result<NewtValue, NewtRuntimeError> {
        self.resolve_at(0, identifier)
    }

    pub fn resolve_at(&self, offset: usize, identifier: &str) -> Result<NewtValue, NewtRuntimeError> {
        self.top.borrow().resolve(identifier)
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
pub struct RefEquality<'a, T: Eq + Hash>(pub &'a T);

impl<'a, T> Hash for RefEquality<'a, T>
    where T: Eq + Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const T).hash(state)
    }
}

impl<'a, 'b, T> PartialEq<RefEquality<'b, T>> for RefEquality<'a, T>
    where T: Eq + Hash
{
    fn eq(&self, other: &RefEquality<'b, T>) -> bool {
        (self.0 as *const T) == (other.0 as *const T)
    }
}

impl<'a, T> Eq for RefEquality<'a, T>
    where T: Eq + Hash
{}
impl<'a, T> From<&'a T> for RefEquality<'a, T>
    where T: Eq + Hash
{
    fn from(reference: &'a T) -> Self {
        RefEquality(reference)
    }
}

#[derive(Debug)]
pub struct LexicalScopeAnalyzer<'a> {
    scopes: Vec<HashMap<String, DeclarationProgress>>,
    resolutions: HashMap<RefEquality<'a, SyntaxNode>, usize>,
    errors: Vec<NewtStaticError>
}

impl<'a> LexicalScopeAnalyzer<'a> {
    pub fn analyze(root: &'a StmtNode) -> Result<HashMap<RefEquality<'a, SyntaxNode>, usize>, Vec<NewtStaticError>> {
        let mut state = LexicalScopeAnalyzer {
            scopes: vec![HashMap::new()],
            resolutions: HashMap::new(),
            errors: Vec::new()
        };

        state.visit_stmt(root);

        if state.errors.is_empty() {
            Ok(state.resolutions)
        } else {
            Err(state.errors)
        }
    }

    fn begin_binding(&mut self, identifier: &str) {
        match self.resolve_binding(identifier) {
            Some(0) => self.errors.push(NewtStaticError::DuplicateVariableDeclaration),
            Some(_) => self.errors.push(NewtStaticError::ShadowedVariableDeclaration),
            None => {}
        }

        self.peek_mut().insert(identifier.to_string(), DeclarationProgress::BeingDeclared);
    }

    fn complete_binding(&mut self, identifier: &str) {
        if let Some(binding) = self.peek_mut().get_mut(identifier) {
            *binding = DeclarationProgress::Declared
        } else {
            panic!("Attempted to complete a binding we never started?");
        }
    }

    fn resolve_binding(&self, identifier: &str) -> Option<usize> {
        let mut offset = 0;
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(identifier) {
                return Some(offset);
            }

            offset = offset + 1;
        }

        return None;
    }

    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn peek(&self) -> &HashMap<String, DeclarationProgress> {
        self.scopes.last().unwrap()
    }

    fn peek_mut(&mut self) -> &mut HashMap<String, DeclarationProgress> {
        self.scopes.last_mut().unwrap()
    }
}

impl<'a> ExprVisitor<'a, ()> for LexicalScopeAnalyzer<'a> {
    fn visit_expr(&mut self, expr: &'a ExprNode) -> () {
        match expr.kind() {
            ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
            ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
            ExprKind::LiteralExpr(node) => self.visit_literal_expr(node),
            ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
            ExprKind::VariableExpr(node) => self.visit_variable_expr(node),
            ExprKind::FunctionCallExpr(node) => self.visit_function_call_expr(node),
        }
    }

    fn visit_binary_expr(&mut self, node: &'a BinaryExprNode) -> () {
        self.visit_expr(node.lhs());
        self.visit_expr(node.rhs());
    }

    fn visit_unary_expr(&mut self, node: &'a UnaryExprNode) -> () {
        self.visit_expr(node.rhs());
    }

    fn visit_literal_expr(&mut self, node: &'a LiteralExprNode) -> () {
        // noop
    }

    fn visit_grouping_expr(&mut self, node: &'a GroupingExprNode) -> () {
        self.visit_expr(node.expr());
    }

    fn visit_variable_expr(&mut self, node: &'a VariableExprNode) -> () {
        if let Some(offset) = self.resolve_binding(node.identifier().lexeme()) {
            self.resolutions.insert(RefEquality(node.to_inner()), offset);
        } else {
            self.errors.push(NewtStaticError::UndeclaredVariable);
        }
    }

    fn visit_function_call_expr(&mut self, node: &'a FunctionCallExprNode) -> () {
        self.visit_expr(node.callee());

        for argument in node.arguments() {
            self.visit_expr(argument);
        }
    }
}

impl<'a> StmtVisitor<'a, Result<(), NewtStaticError>> for LexicalScopeAnalyzer<'a> {
    fn visit_stmt(&mut self, stmt: &'a StmtNode) -> Result<(), NewtStaticError> {
        match stmt.kind() {
            StmtKind::VariableDeclarationStmt(node) => self.visit_variable_declaration_stmt(node),
            StmtKind::VariableAssignmentStmt(node) => self.visit_variable_assignment_stmt(node),
            StmtKind::StmtListStmt(node) => self.visit_stmt_list_stmt(node),
            StmtKind::ExprStmt(node) => self.visit_expr_stmt(node),
            StmtKind::IfStmt(node) => self.visit_if_stmt(node),
            StmtKind::WhileStmt(node) => self.visit_while_stmt(node),
            StmtKind::FunctionDeclarationStmt(node) => self.visit_function_declaration_stmt(node),
            StmtKind::ReturnStmt(node) => self.visit_return_stmt(node),
        }
    }

    fn visit_variable_declaration_stmt(&mut self, node: &'a VariableDeclarationStmtNode) -> Result<(), NewtStaticError> {
        self.begin_binding(node.identifier().lexeme());
        self.visit_expr(node.expr());
        self.complete_binding(node.identifier().lexeme());

        Ok(())
    }

    fn visit_variable_assignment_stmt(&mut self, node: &'a VariableAssignmentStmtNode) -> Result<(), NewtStaticError> {
        let offset = self.resolve_binding(node.identifier().lexeme())
            .ok_or(NewtStaticError::UndeclaredVariable)?;
        self.resolutions.insert(node.to_inner().into(), offset);
        self.visit_expr(node.expr());

        Ok(())
    }

    fn visit_stmt_list_stmt(&mut self, node: &'a StmtListStmtNode) -> Result<(), NewtStaticError> {
        self.push();

        for stmt in node.stmts() {
            self.visit_stmt(stmt);
        }

        self.pop();

        Ok(())
    }

    fn visit_expr_stmt(&mut self, node: &'a ExprStmtNode) -> Result<(), NewtStaticError> {
        self.visit_expr(node.expr());

        Ok(())
    }

    fn visit_if_stmt(&mut self, node: &'a IfStmtNode) -> Result<(), NewtStaticError> {
        self.visit_expr(node.condition());
        self.visit_stmt_list_stmt(node.when_true());

        if let Some(falsey) = node.when_false() {
            self.visit_stmt_list_stmt(falsey);
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, node: &'a WhileStmtNode) -> Result<(), NewtStaticError> {
        self.visit_expr(node.condition());
        self.visit_stmt_list_stmt(node.stmts());

        Ok(())
    }

    fn visit_function_declaration_stmt(&mut self, node: &'a FunctionDeclarationStmtNode) -> Result<(), NewtStaticError> {
        self.begin_binding(node.identifier().lexeme());
        self.complete_binding(node.identifier().lexeme());

        Ok(())
    }

    fn visit_return_stmt(&mut self, node: &'a ReturnStmtNode) -> Result<(), NewtStaticError> {
        if let Some(result) = node.result() {
            self.visit_expr(result);
        }

        Ok(())
    }
}

mod lexical_scope_analyzer_tests {
    use crate::featurez::runtime::scope::{LexicalScope, LexicalScopeAnalyzer, RefEquality};
    use crate::featurez::syntax::{NewtValue, NewtRuntimeError, SyntaxToken, SyntaxTree, StmtNode, AstNode, SyntaxElement, SyntaxNode, WhileStmtNode, SyntaxKind, NewtStaticError, VariableExprNode, VariableAssignmentStmtNode};
    use crate::featurez::grammar::root_stmt;
    use crate::featurez::{InterpretingSession, InterpretingSessionKind};
    use crate::featurez::newtypes::TransparentNewType;
    use std::collections::HashMap;
    use std::borrow::Borrow;

    #[test]
    pub fn variable_in_condition_resolves_to_same_scope()
    {
        let session = source_to_tree("{
                let x = 42;
                while x > 0 {
                    x = x + 1;
                }
            }");
        let resolutions = tree_to_resolutions(session.syntax_tree())
            .expect("source is valid");
        let vars = tree_to_variable_references(session.syntax_tree(), "x");

        assert_eq!(3, resolutions.len());
        assert_eq!(2, vars.len());
        assert_eq!(0, resolutions[&RefEquality(vars[0])]);
        assert_eq!(1, resolutions[&RefEquality(vars[1])]);
    }

    #[test]
    pub fn variable_declared_in_scope_0_used_in_scope_1_resolves_to_scope_0()
    {
        let session = source_to_tree("{
                let x = 42;
                {
                    x = x + 1;
                }
            }");
        let resolutions = tree_to_resolutions(session.syntax_tree())
            .expect("source is valid");
        let x_references = tree_to_variable_references(session.syntax_tree(), "x");

        assert_eq!(2, resolutions.len());
        assert_eq!(1, x_references.len());
        assert_eq!(1, resolutions[&RefEquality(x_references[0])]);
    }

    #[test]
    pub fn variable_resolution_resolves_correct_variable()
    {
        let session = source_to_tree("{
                let x = 42;
                {
                    let y = 32;
                    x = y + 1;
                }
            }");
        let resolutions = tree_to_resolutions(session.syntax_tree())
            .expect("source is valid");
        let x_references = tree_to_variable_assignments(session.syntax_tree(), "x");
        let y_references = tree_to_variable_references(session.syntax_tree(), "y");

        assert_eq!(2, resolutions.len());
        assert_eq!(1, x_references.len());
        assert_eq!(1, y_references.len());
        assert_eq!(1, resolutions[&RefEquality(x_references[0])]);
        assert_eq!(0, resolutions[&RefEquality(y_references[0])]);
    }

	#[test]
	pub fn variable_resolution_reports_undeclared_variables()
	{
		let session = source_to_tree("let y = 32 + x;");
		let errors = tree_to_resolutions(session.syntax_tree())
            .err()
            .expect("Source is invalid");

		assert_eq!(1, errors.len());
        assert_eq!(NewtStaticError::UndeclaredVariable, errors[0]);
	}

    #[test]
    pub fn variable_resolution_reports_duplicate_variables()
    {
        let session = source_to_tree("{
            let x = 42;
            let x = 2;
        }");
        let errors = tree_to_resolutions(session.syntax_tree())
            .err()
            .expect("Source is invalid");

        assert_eq!(1, errors.len());
        assert_eq!(NewtStaticError::DuplicateVariableDeclaration, errors[0]);
    }

    #[test]
    pub fn variable_resolution_reports_shadowed_variables()
    {
        let session = source_to_tree("{
            let x = 42;
            {
                let x = 32;
            }
        }");
        let errors = tree_to_resolutions(session.syntax_tree())
            .err()
            .expect("Source is invalid");

        assert_eq!(1, errors.len());
        assert_eq!(NewtStaticError::ShadowedVariableDeclaration, errors[0]);
    }

    fn source_to_tree(source: &str) -> InterpretingSession {
        InterpretingSession::new(InterpretingSessionKind::Stmt, source)
    }

    fn tree_to_resolutions<'a>(tree: &'a SyntaxTree)
        -> Result<HashMap<RefEquality<'a, SyntaxNode>, usize>, Vec<NewtStaticError>> {
        let root = StmtNode::cast(tree.root().as_node().unwrap()).unwrap();
        LexicalScopeAnalyzer::analyze(root)
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

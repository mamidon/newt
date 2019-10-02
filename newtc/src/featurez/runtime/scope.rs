use crate::featurez::syntax::{NewtResult, ExprVisitor};
use crate::featurez::syntax::{NewtRuntimeError, NewtValue};
use crate::featurez::syntax::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::hash::{Hash, Hasher};

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
struct RefEquality<'a, T>(&'a T);

impl<'a, T> Hash for RefEquality<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const T).hash(state)
    }
}

impl<'a, 'b, T> PartialEq<RefEquality<'b, T>> for RefEquality<'a, T> {
    fn eq(&self, other: &RefEquality<'b, T>) -> bool {
        (self.0 as *const T) == (other.0 as *const T)
    }
}

impl<'a, T> Eq for RefEquality<'a, T> {}
impl<'a, T> From<&'a T> for RefEquality<'a, T> {
    fn from(reference: &'a T) -> Self {
        RefEquality(reference)
    }
}

#[derive(Debug)]
struct LexicalScopeAnalyzer {
    scopes: Vec<HashMap<String, DeclarationProgress>>,
    resolutions: HashMap<usize, usize>,
    errors: Vec<NewtStaticError>
}

impl LexicalScopeAnalyzer {
    pub fn new() -> LexicalScopeAnalyzer {
        LexicalScopeAnalyzer {
            scopes: vec![HashMap::new()],
            resolutions: HashMap::new(),
            errors: Vec::new()
        }
    }

    fn ref_to_key<T>(&self, reference: &T) -> usize {
        (reference as *const T) as usize
    }

    pub fn analyze(&mut self, root: &StmtNode) {
        self.visit_stmt(root);

        //self.resolutions
    }

    fn begin_binding(&mut self, identifier: &str) {
        if self.peek().contains_key(identifier) {
            self.errors.push(NewtStaticError::DuplicateVariableDeclaration);
            return;
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

impl ExprVisitor<()> for LexicalScopeAnalyzer {
    fn visit_expr(&mut self, expr: &ExprNode) -> () {
        match expr.kind() {
            ExprKind::BinaryExpr(node) => self.visit_binary_expr(node),
            ExprKind::UnaryExpr(node) => self.visit_unary_expr(node),
            ExprKind::LiteralExpr(node) => self.visit_literal_expr(node),
            ExprKind::GroupingExpr(node) => self.visit_grouping_expr(node),
            ExprKind::VariableExpr(node) => self.visit_variable_expr(node),
            ExprKind::FunctionCallExpr(node) => self.visit_function_call_expr(node),
        }
    }

    fn visit_binary_expr(&mut self, node: &BinaryExprNode) -> () {
        self.visit_expr(node.lhs());
        self.visit_expr(node.rhs());
    }

    fn visit_unary_expr(&mut self, node: &UnaryExprNode) -> () {
        self.visit_expr(node.rhs());
    }

    fn visit_literal_expr(&mut self, node: &LiteralExprNode) -> () {
        // noop
    }

    fn visit_grouping_expr(&mut self, node: &GroupingExprNode) -> () {
        self.visit_expr(node.expr());
    }

    fn visit_variable_expr(&mut self, node: &VariableExprNode) -> () {
        if let Some(offset) = self.resolve_binding(node.identifier().lexeme()) {
            self.resolutions.insert(self.ref_to_key(node), offset);
        } else {
            self.errors.push(NewtStaticError::UndeclaredVariable);
        }
    }

    fn visit_function_call_expr(&mut self, node: &FunctionCallExprNode) -> () {
        self.visit_expr(node.callee());

        for argument in node.arguments() {
            self.visit_expr(argument);
        }
    }
}

impl StmtVisitor<Result<(), NewtStaticError>> for LexicalScopeAnalyzer {
    fn visit_stmt(&mut self, stmt: &StmtNode) -> Result<(), NewtStaticError> {
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

    fn visit_variable_declaration_stmt(&mut self, node: &VariableDeclarationStmtNode) -> Result<(), NewtStaticError> {
        self.begin_binding(node.identifier().lexeme());
        self.visit_expr(node.expr());
        self.complete_binding(node.identifier().lexeme());

        Ok(())
    }

    fn visit_variable_assignment_stmt(&mut self, node: &VariableAssignmentStmtNode) -> Result<(), NewtStaticError> {
        let offset = self.resolve_binding(node.identifier().lexeme())
            .ok_or(NewtStaticError::UndeclaredVariable)?;
        self.resolutions.insert(self.ref_to_key(node), offset);
        self.visit_expr(node.expr());

        Ok(())
    }

    fn visit_stmt_list_stmt(&mut self, node: &StmtListStmtNode) -> Result<(), NewtStaticError> {
        self.push();

        for stmt in node.stmts() {
            self.visit_stmt(stmt);
        }

        self.pop();

        Ok(())
    }

    fn visit_expr_stmt(&mut self, node: &ExprStmtNode) -> Result<(), NewtStaticError> {
        self.visit_expr(node.expr());

        Ok(())
    }

    fn visit_if_stmt(&mut self, node: &IfStmtNode) -> Result<(), NewtStaticError> {
        self.visit_expr(node.condition());
        self.visit_stmt_list_stmt(node.when_true());

        if let Some(falsey) = node.when_false() {
            self.visit_stmt_list_stmt(falsey);
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, node: &WhileStmtNode) -> Result<(), NewtStaticError> {
        self.visit_expr(node.condition());
        self.visit_stmt_list_stmt(node.stmts());

        Ok(())
    }

    fn visit_function_declaration_stmt(&mut self, node: &FunctionDeclarationStmtNode) -> Result<(), NewtStaticError> {
        self.begin_binding(node.identifier().lexeme());
        self.complete_binding(node.identifier().lexeme());

        Ok(())
    }

    fn visit_return_stmt(&mut self, node: &ReturnStmtNode) -> Result<(), NewtStaticError> {
        if let Some(result) = node.result() {
            self.visit_expr(result);
        }

        Ok(())
    }
}

mod tests {
    use crate::featurez::runtime::scope::{LexicalScope, LexicalScopeAnalyzer};
    use crate::featurez::syntax::{NewtValue, NewtRuntimeError, SyntaxToken, SyntaxTree, StmtNode, AstNode};
    use crate::featurez::grammar::root_stmt;

    #[test]
    pub fn foo()
    {
        use crate::featurez::{tokenize, StrTokenSource};
        use crate::featurez::parse::Parser;

        let source = "{ let x = 42;\
        while x > 0 {\
         x = x + 1;\
        }}";

        let tokens = tokenize(source);
        let token_source = StrTokenSource::new(tokens);
        let mut parser = Parser::new(token_source);
        let tree = SyntaxTree::from_parser(&root_stmt(parser), source);
        println!("{:#?}", tree);
        let mut analyzer = LexicalScopeAnalyzer::new();
        analyzer.analyze(StmtNode::cast(tree.root().as_node().unwrap()).unwrap());

        println!("{:#?}", analyzer);
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

use crate::featurez::newtypes::TransparentNewType;
use crate::featurez::syntax::{AstNode, ExprKind, StmtKind, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, RValKind};
use crate::featurez::tokens::{Token, TokenKind};

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::rc::Rc;
use std::collections::HashMap;

#[repr(transparent)]
#[derive(Clone)]
pub struct StmtNode(SyntaxNode);

unsafe impl TransparentNewType for StmtNode {
    type Inner = SyntaxNode;
}

impl AstNode for StmtNode {
    fn cast(node: &SyntaxNode) -> Option<&Self> {
        match node.kind() {
            SyntaxKind::VariableDeclarationStmt
            | SyntaxKind::AssignmentStmt
            | SyntaxKind::ExprStmt
            | SyntaxKind::IfStmt
            | SyntaxKind::StmtListStmt
            | SyntaxKind::WhileStmt
            | SyntaxKind::FunctionDeclarationStmt
            | SyntaxKind::ReturnStmt => Some(StmtNode::from_inner(node)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        self.to_inner()
    }
}

impl StmtNode {
    pub fn kind(&self) -> StmtKind {
        match self.syntax().kind() {
            SyntaxKind::VariableDeclarationStmt => StmtKind::VariableDeclarationStmt(
                VariableDeclarationStmtNode::from_inner(self.syntax()),
            ),
            SyntaxKind::AssignmentStmt => StmtKind::AssignmentStmt(
                AssignmentStmtNode::from_inner(self.syntax()),
            ),
            SyntaxKind::ExprStmt => StmtKind::ExprStmt(ExprStmtNode::from_inner(self.syntax())),
            SyntaxKind::IfStmt => StmtKind::IfStmt(IfStmtNode::from_inner(self.syntax())),
            SyntaxKind::WhileStmt => StmtKind::WhileStmt(WhileStmtNode::from_inner(self.syntax())),
            SyntaxKind::FunctionDeclarationStmt => StmtKind::FunctionDeclarationStmt(
                FunctionDeclarationStmtNode::from_inner(self.syntax()),
            ),
            SyntaxKind::ReturnStmt => {
                StmtKind::ReturnStmt(ReturnStmtNode::from_inner(self.syntax()))
            },
            SyntaxKind::StmtListStmt => {
                StmtKind::StmtListStmt(StmtListStmtNode::from_inner(self.syntax()))
            },
            _ => unreachable!("StmtNode cannot be constructed from invalid SyntaxKind"),
        }
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ReturnStmtNode(SyntaxNode);

unsafe impl TransparentNewType for ReturnStmtNode {
    type Inner = SyntaxNode;
}

impl ReturnStmtNode {
    pub fn result(&self) -> Option<&ExprNode> {
        self.0.try_nth_node(0).map(|n| ExprNode::from_inner(n))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct FunctionDeclarationStmtNode(SyntaxNode);

unsafe impl TransparentNewType for FunctionDeclarationStmtNode {
    type Inner = SyntaxNode;
}

impl FunctionDeclarationStmtNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(1)
    }

    pub fn arguments(&self) -> impl Iterator<Item = &SyntaxToken> {
	    self.0.tokens()
		    .filter(|t| t.token_kind() == TokenKind::Identifier)
		    .skip(1)
    }

    pub fn stmts(&self) -> &StmtListStmtNode {
        let node = self.0.nodes().last().expect("Expecting StmtListStmtNode");
        StmtListStmtNode::from_inner(node)
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct WhileStmtNode(SyntaxNode);

unsafe impl TransparentNewType for WhileStmtNode {
    type Inner = SyntaxNode;
}

impl WhileStmtNode {
    pub fn condition(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0))
            .expect("Expected an expression node for the while statement's condition")
    }

    pub fn stmts(&self) -> &StmtListStmtNode {
        StmtListStmtNode::from_inner(self.0.nth_node(1))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct IfStmtNode(SyntaxNode);

unsafe impl TransparentNewType for IfStmtNode {
    type Inner = SyntaxNode;
}

impl IfStmtNode {
    pub fn condition(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0))
            .expect("Expected an expression node for an if statement's condition")
    }

    pub fn when_true(&self) -> &StmtListStmtNode {
        StmtListStmtNode::from_inner(self.0.nth_node(1))
    }

    pub fn when_false(&self) -> Option<&StmtListStmtNode> {
        self.0
            .try_nth_node(2)
            .map(|c| StmtListStmtNode::from_inner(c))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct VariableDeclarationStmtNode(SyntaxNode);

unsafe impl TransparentNewType for VariableDeclarationStmtNode {
    type Inner = SyntaxNode;
}

impl VariableDeclarationStmtNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0
            .children()
            .iter()
            .filter_map(|c| c.as_token())
            .filter(|c| c.token_kind() == TokenKind::Identifier)
            .nth(0)
            .unwrap()
    }

    pub fn expr(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0))
            .expect("Expected an expression node in variable declaration statement")
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct AssignmentStmtNode(SyntaxNode);

unsafe impl TransparentNewType for AssignmentStmtNode {
    type Inner = SyntaxNode;
}

impl AssignmentStmtNode {
    pub fn rval(&self) -> &RValNode {
        RValNode::from_inner(self.0.nth_node(0))
    }

    pub fn expr(&self) -> &ExprNode {
        ExprNode::from_inner(self.0.nth_node(1))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct StmtListStmtNode(SyntaxNode);

unsafe impl TransparentNewType for StmtListStmtNode {
    type Inner = SyntaxNode;
}

impl StmtListStmtNode {
    pub fn stmts(&self) -> impl Iterator<Item = &StmtNode> {
        self.0
            .children()
            .iter()
            .filter_map(|n| n.as_node())
            .filter_map(StmtNode::cast)
    }

    pub fn has_braces(&self) -> bool {
        let token_count = self.0.tokens().count();

        if token_count > 0 {
            let first = self.0.tokens().nth(0).unwrap();
            let last = self.0.tokens().nth(token_count - 1).unwrap();

            first.token_kind() == TokenKind::LeftBrace && last.token_kind() == TokenKind::RightBrace
        } else {
            false
        }
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ExprStmtNode(SyntaxNode);

unsafe impl TransparentNewType for ExprStmtNode {
    type Inner = SyntaxNode;
}

impl ExprStmtNode {
    pub fn expr(&self) -> &ExprNode {
        ExprNode::from_inner(self.0.nth_node(0))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ExprNode(SyntaxNode);

unsafe impl TransparentNewType for ExprNode {
    type Inner = SyntaxNode;
}

impl<'a> From<&'a BinaryExprNode> for &'a ExprNode {
    fn from(node: &'a BinaryExprNode) -> Self {
        ExprNode::from_inner(node.to_inner())
    }
}

impl AstNode for ExprNode {
    fn cast(node: &SyntaxNode) -> Option<&Self> {
        match node.kind() {
            SyntaxKind::BinaryExpr
            | SyntaxKind::UnaryExpr
            | SyntaxKind::PrimitiveLiteralExpr
            | SyntaxKind::GroupingExpr
            | SyntaxKind::VariableExpr
            | SyntaxKind::FunctionCallExpr
            | SyntaxKind::ObjectLiteralExpr
            | SyntaxKind::ObjectPropertyExpr => Some(ExprNode::from_inner(node)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        self.to_inner()
    }
}

impl ExprNode {
    pub fn kind(&self) -> ExprKind {
        match self.syntax().kind() {
            SyntaxKind::BinaryExpr => {
                ExprKind::BinaryExpr(BinaryExprNode::from_inner(self.to_inner()))
            }
            SyntaxKind::UnaryExpr => {
                ExprKind::UnaryExpr(UnaryExprNode::from_inner(self.to_inner()))
            }
            SyntaxKind::PrimitiveLiteralExpr => {
                ExprKind::PrimitiveLiteralExpr(PrimitiveLiteralExprNode::from_inner(self.to_inner()))
            }
	        SyntaxKind::ObjectLiteralExpr => {
		        ExprKind::ObjectLiteralExpr(ObjectLiteralExprNode::from_inner(self.to_inner()))
	        }
            SyntaxKind::ObjectPropertyExpr => {
                ExprKind::ObjectPropertyExpr(ObjectPropertyExprNode::from_inner(self.to_inner()))
            }
            SyntaxKind::GroupingExpr => {
                ExprKind::GroupingExpr(GroupingExprNode::from_inner(self.to_inner()))
            }
            SyntaxKind::VariableExpr => {
                ExprKind::VariableExpr(VariableExprNode::from_inner(self.to_inner()))
            }
            SyntaxKind::FunctionCallExpr => {
                ExprKind::FunctionCallExpr(FunctionCallExprNode::from_inner(self.to_inner()))
            }
            _ => unreachable!("ExprNode cannot be constructed from invalid SyntaxKind"),
        }
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct FunctionCallExprNode(SyntaxNode);

unsafe impl TransparentNewType for FunctionCallExprNode {
    type Inner = SyntaxNode;
}

impl FunctionCallExprNode {
    pub fn callee(&self) -> &ExprNode {
        ExprNode::from_inner(self.0.nth_node(0))
    }

    pub fn arguments(&self) -> impl Iterator<Item = &ExprNode> {
        self.0
	        .nodes()
            .filter_map(|n| ExprNode::cast(n))
	        .skip(1)
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct PrimitiveLiteralExprNode(SyntaxNode);

unsafe impl TransparentNewType for PrimitiveLiteralExprNode {
    type Inner = SyntaxNode;
}

impl PrimitiveLiteralExprNode {
    pub fn literal(&self) -> &SyntaxToken {
        self.0.nth_token(0)
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ObjectLiteralExprNode(SyntaxNode);

unsafe impl TransparentNewType for ObjectLiteralExprNode {
    type Inner = SyntaxNode;
}

impl ObjectLiteralExprNode {
    pub fn fields(&self) -> HashMap<String, ExprNode> {
        let relevant_children: Vec<_> = self.0
            .children()
            .iter()
            .filter(ObjectLiteralExprNode::is_identifier_token_or_expr_node)
            .collect();
        let pairs = relevant_children.chunks_exact(2);
        let mut map: HashMap<String, ExprNode> = HashMap::new();

        if !pairs.remainder().is_empty() {
            panic!("Object literal did not have fully formed pairs");
        }

        for slice in pairs {
            match slice {
                [key_element, value_element] => {
                    let key = ObjectLiteralExprNode::as_identifier(key_element).unwrap().lexeme().to_string();
                    let value = ObjectLiteralExprNode::as_expr_node(value_element).unwrap().clone();
                    map.insert(key, value);
                }
                _ => unreachable!("Shouldn't happen with chunks_exact of 2")
            }
        }

        map
    }

    fn as_identifier(element: &&SyntaxElement) -> Option<SyntaxToken> {
        match element {
            SyntaxElement::Token(token) => Some(token.clone()),
            _ => None
        }
    }

    fn as_expr_node(element: &SyntaxElement) -> Option<&ExprNode> {
        match element {
            SyntaxElement::Node(node) => ExprNode::cast(node),
            _ => None
        }
    }

    fn is_identifier_token_or_expr_node(element: &&SyntaxElement) -> bool {
        match element {
            SyntaxElement::Token(token) => token.token_kind() == TokenKind::Identifier,
            SyntaxElement::Node(node) => ExprNode::cast(node).is_some()
        }
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ObjectPropertyExprNode(SyntaxNode);

unsafe impl TransparentNewType for ObjectPropertyExprNode {
    type Inner = SyntaxNode;
}

impl ObjectPropertyExprNode {
	pub fn source_expr(&self) -> &ExprNode {
		ExprNode::cast(self.0.nth_node(0)).unwrap()
	}

    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(1)
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct BinaryExprNode(SyntaxNode);

unsafe impl TransparentNewType for BinaryExprNode {
    type Inner = SyntaxNode;
}

impl BinaryExprNode {
    pub fn operator(&self) -> TokenKind {
        self.0.nth_token(0).token_kind()
    }

    pub fn lhs(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0)).unwrap()
    }

    pub fn rhs(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(1)).unwrap()
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct UnaryExprNode(SyntaxNode);

unsafe impl TransparentNewType for UnaryExprNode {
    type Inner = SyntaxNode;
}

impl UnaryExprNode {
    pub fn operator(&self) -> TokenKind {
        self.0.nth_token(0).token_kind()
    }

    pub fn rhs(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0)).unwrap()
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct GroupingExprNode(SyntaxNode);

unsafe impl TransparentNewType for GroupingExprNode {
    type Inner = SyntaxNode;
}

impl GroupingExprNode {
    pub fn expr(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0)).unwrap()
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct VariableExprNode(SyntaxNode);

unsafe impl TransparentNewType for VariableExprNode {
    type Inner = SyntaxNode;
}

impl VariableExprNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(0)
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct RValNode(SyntaxNode);

unsafe impl TransparentNewType for RValNode {
    type Inner = SyntaxNode;
}

impl AstNode for RValNode {
    fn cast(node: &SyntaxNode) -> Option<&Self> {
        match node.kind() {
            SyntaxKind::ObjectPropertyRVal
            | SyntaxKind::VariableRval => {
                Some(RValNode::from_inner(node))
            }
            _ => None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RValNode {
    pub fn kind(&self) -> RValKind {
        match self.0.kind() {
            SyntaxKind::VariableRval
                => RValKind::VariableRVal(VariableRValNode::from_inner(&self.0)),
            SyntaxKind::ObjectPropertyRVal
                => RValKind::ObjectPropertyRVal(ObjectPropertyRValNode::from_inner(&self.0)),
            kind => unreachable!("An RValNode should not contain an {:?} node", kind)
        }
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct VariableRValNode(SyntaxNode);

unsafe impl TransparentNewType for VariableRValNode {
    type Inner = SyntaxNode;
}

impl VariableRValNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(0)
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ObjectPropertyRValNode(SyntaxNode);

unsafe impl TransparentNewType for ObjectPropertyRValNode {
    type Inner = SyntaxNode;
}

impl ObjectPropertyRValNode {
    pub fn source_expr(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0)).unwrap()
    }

    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(1)
    }
}




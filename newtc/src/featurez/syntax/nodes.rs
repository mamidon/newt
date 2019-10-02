use crate::featurez::newtypes::TransparentNewType;
use crate::featurez::syntax::{
    AstNode, ExprKind, StmtKind, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};
use crate::featurez::tokens::{Token, TokenKind};

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::rc::Rc;

#[repr(transparent)]
pub struct StmtNode(SyntaxNode);

unsafe impl TransparentNewType for StmtNode {
    type Inner = SyntaxNode;
}

impl AstNode for StmtNode {
    fn cast(node: &SyntaxNode) -> Option<&Self> {
        match node.kind() {
            SyntaxKind::VariableDeclarationStmt
            | SyntaxKind::VariableAssignmentStmt
            | SyntaxKind::ExprStmt
            | SyntaxKind::IfStmt
            | SyntaxKind::StmtListStmt
            | SyntaxKind::WhileStmt => Some(StmtNode::from_inner(node)),
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
            SyntaxKind::VariableAssignmentStmt => StmtKind::VariableAssignmentStmt(
                VariableAssignmentStmtNode::from_inner(self.syntax()),
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
pub struct FunctionDeclarationStmtNode(SyntaxNode);

unsafe impl TransparentNewType for FunctionDeclarationStmtNode {
    type Inner = SyntaxNode;
}

impl FunctionDeclarationStmtNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(1)
    }

    pub fn arguments(&self) -> impl Iterator<Item = &SyntaxToken> {
        self.0.children().iter().filter_map(|e| e.as_token())
    }

    pub fn stmts(&self) -> &StmtListStmtNode {
        let node = self.0.nodes().last().expect("Expecting StmtListStmtNode");
        StmtListStmtNode::from_inner(node)
    }
}

#[repr(transparent)]
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
pub struct VariableAssignmentStmtNode(SyntaxNode);

unsafe impl TransparentNewType for VariableAssignmentStmtNode {
    type Inner = SyntaxNode;
}

impl VariableAssignmentStmtNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(0)
    }

    pub fn expr(&self) -> &ExprNode {
        ExprNode::cast(self.0.nth_node(0))
            .expect("Expected an expression node in variable declaration statement")
    }
}

#[repr(transparent)]
pub struct StmtListStmtNode(SyntaxNode);

unsafe impl TransparentNewType for StmtListStmtNode {
    type Inner = SyntaxNode;
}

impl StmtListStmtNode {
    pub fn stmts(&self) -> impl IntoIterator<Item = &StmtNode> {
        self.0
            .children()
            .iter()
            .filter_map(|n| n.as_node())
            .filter_map(StmtNode::cast)
    }
}

#[repr(transparent)]
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
            | SyntaxKind::LiteralExpr
            | SyntaxKind::GroupingExpr
            | SyntaxKind::VariableExpr => Some(ExprNode::from_inner(node)),
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
            SyntaxKind::LiteralExpr => {
                ExprKind::LiteralExpr(LiteralExprNode::from_inner(self.to_inner()))
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
            .children()
            .iter()
            .filter_map(|c| c.as_node())
            .filter_map(|n| ExprNode::cast(n))
    }
}

#[repr(transparent)]
pub struct LiteralExprNode(SyntaxNode);

unsafe impl TransparentNewType for LiteralExprNode {
    type Inner = SyntaxNode;
}

impl LiteralExprNode {
    pub fn literal(&self) -> &SyntaxToken {
        self.0.nth_token(0)
    }
}

#[repr(transparent)]
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
pub struct VariableExprNode(SyntaxNode);

unsafe impl TransparentNewType for VariableExprNode {
    type Inner = SyntaxNode;
}

impl VariableExprNode {
    pub fn identifier(&self) -> &SyntaxToken {
        self.0.nth_token(0)
    }
}

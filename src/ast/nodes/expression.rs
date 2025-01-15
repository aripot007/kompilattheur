use crate::{
    common::types::{FileElement, Node, Tree},
    parser::Lexem,
};

use super::{AstNode, Factor};

pub enum Expression {
    OR(Box<Expression>, Box<Expression>),
    AND(Box<Expression>, Box<Expression>),
    NOT(Box<Expression>),
    CMP(Box<Expression>, CmpOp, Box<Expression>),
    ADD(Box<Expression>, AddOp, Box<Expression>),
    MULT(Box<Expression>, MultOp, Box<Expression>),
    NEG(Box<Expression>),
    Factor(Factor),
    NotImplemented,
}

#[derive(Debug, Copy, Clone)]
pub enum CmpOp {
    LESS,
    LESSEQ,
    GREATER,
    GREATEREQ,
    EQ,
    NEQ,
}
#[derive(Debug, Copy, Clone)]
pub enum AddOp {
    ADD,
    SUB,
}
#[derive(Debug, Copy, Clone)]
pub enum MultOp {
    MULT,
    DIV,
    MOD,
}

impl Into<String> for CmpOp {
    fn into(self) -> String {
        String::from(match self {
            CmpOp::LESS => "<",
            CmpOp::LESSEQ => "<=",
            CmpOp::GREATER => ">",
            CmpOp::GREATEREQ => ">=",
            CmpOp::EQ => "==",
            CmpOp::NEQ => "!=",
        })
    }
}

impl Into<String> for AddOp {
    fn into(self) -> String {
        String::from(match self {
            AddOp::ADD => "+",
            AddOp::SUB => "-",
        })
    }
}

impl Into<String> for MultOp {
    fn into(self) -> String {
        String::from(match self {
            MultOp::MULT => "*",
            MultOp::DIV => "//",
            MultOp::MOD => "%",
        })
    }
}

impl AstNode for Expression {}

impl From<Tree<FileElement<Lexem>>> for Expression {
    fn from(_root: Tree<FileElement<Lexem>>) -> Self {
        return Expression::NotImplemented;
    }
}

impl Into<Tree<String>> for Expression {
    fn into(self) -> Tree<String> {
        (&self).into()
    }
}

impl Into<Tree<String>> for &Expression {
    fn into(self) -> Tree<String> {
        match self {
            Expression::OR(e1, e2) => {
                let root = Node::new(String::from("OR"));
                root.borrow_mut().add_child(&root, e1.as_ref().into());
                root.borrow_mut().add_child(&root, e2.as_ref().into());
                root
            }
            Expression::AND(e1, e2) => {
                let root = Node::new(String::from("AND"));
                root.borrow_mut().add_child(&root, e1.as_ref().into());
                root.borrow_mut().add_child(&root, e2.as_ref().into());
                root
            }
            Expression::CMP(e1, op, e2) => {
                let root = Node::new((*op).into());
                root.borrow_mut().add_child(&root, e1.as_ref().into());
                root.borrow_mut().add_child(&root, e2.as_ref().into());
                root
            }
            Expression::ADD(e1, op, e2) => {
                let root = Node::new((*op).into());
                root.borrow_mut().add_child(&root, e1.as_ref().into());
                root.borrow_mut().add_child(&root, e2.as_ref().into());
                root
            }
            Expression::MULT(e1, op, e2) => {
                let root = Node::new((*op).into());
                root.borrow_mut().add_child(&root, e1.as_ref().into());
                root.borrow_mut().add_child(&root, e2.as_ref().into());
                root
            }
            Expression::NEG(expression) => {
                let root = Node::new(String::from("NEG"));
                root.borrow_mut()
                    .add_child(&root, expression.as_ref().into());
                root
            }
            Expression::NOT(expression) => {
                let root = Node::new(String::from("NOT"));
                root.borrow_mut()
                    .add_child(&root, expression.as_ref().into());
                root
            }
            Expression::Factor(factor) => factor.into(),
            Expression::NotImplemented => Node::new(String::from("EXPR (NI)")),
        }
    }
}

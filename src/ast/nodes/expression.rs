use crate::{common::types::{FileElement, Node, Tree}, parser::Lexem};

use super::AstNode;


pub enum Expression {
    NotImplemented,
}

impl AstNode for Expression {}

impl From<Tree<FileElement<Lexem>>> for Expression {
    fn from(_root: Tree<FileElement<Lexem>>) -> Self {
        return Expression::NotImplemented;
    }
}

impl Into<Tree<String>> for Expression {
    fn into(self) -> Tree<String> {
        let s = match self {
            Expression::NotImplemented =>  String::from("EXPR"),
        };
        
        let root = Node::new(s);
        
        return root;
    }
}

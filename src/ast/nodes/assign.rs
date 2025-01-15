use super::AstNode;
use crate::{
    common::types::{FileElement, Node, Tree},
    parser::Lexem,
};

use super::{Expression, Memory};

pub struct Assign {
    memory: Memory,
    expression: Expression,
}

impl AstNode for Assign {}

impl From<Tree<FileElement<Lexem>>> for Assign {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let memory = Memory::from(root.clone());

        let expression = Expression::from(
            root.borrow() // simple statement
                .get_children()[1]
                .borrow() // simple statement identifier
                .get_children()[1]
                .clone(),
        );

        return Assign { memory, expression };
    }
}

impl Into<Tree<String>> for Assign {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("ASSIGN"));

        root.borrow_mut().add_child(&root, self.memory.into());
        root.borrow_mut().add_child(&root, self.expression.into());

        return root;
    }
}

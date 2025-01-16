use super::AstNode;
use crate::{
    common::types::{FileElement, Node, Tree},
    parser::Lexem,
};

use super::Expression;

pub struct Assign {
    destination: Expression,
    value: Expression,
}

impl AstNode for Assign {}

impl From<Tree<FileElement<Lexem>>> for Assign {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let dest = Expression::from(root.borrow().get_children()[0].clone());

        let value = Expression::from(
            root.borrow() // simple statement
                .get_children()[1]
                .borrow() // simple statement identifier
                .get_children()[1]
                .clone(),
        );

        return Assign { destination: dest, value };
    }
}

impl Into<Tree<String>> for Assign {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("ASSIGN"));

        root.borrow_mut().add_child(&root, self.destination.into());
        root.borrow_mut().add_child(&root, self.value.into());

        return root;
    }
}

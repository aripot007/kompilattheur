use super::AstNode;
use crate::{
    common::{localizable::Localizable, types::{FileElement, Node, Tree}},
    parser::Lexem,
};

use super::Expression;

pub struct Assign {
    pub destination: Expression,
    pub value: Expression,
}

impl AstNode for Assign {}

impl Assign {
    pub fn new(destination: Expression, value: Expression) -> Assign {
        return Assign { destination, value };
    }
}

impl From<Tree<FileElement<Lexem>>> for Assign {
    fn from(_root: Tree<FileElement<Lexem>>) -> Self {
        panic!("Assign from tree is not implemented !");
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

impl Localizable for Assign {
    fn get_start_line(&self) -> usize {
        todo!()
    }

    fn get_end_line(&self) -> usize {
        todo!()
    }

    fn get_start_char(&self) -> usize {
        todo!()
    }

    fn get_end_char(&self) -> usize {
        todo!()
    }
}

use super::AstNode;
use crate::{
    common::{
        localizable::Localizable,
        types::{FileElement, Node, Tree},
    },
    parser::Lexem,
};

use super::Expression;

pub struct Assign {
    pub destination: Expression,
    pub value: Expression,
    pub localization: FileElement<bool>,
}

impl AstNode for Assign {}

impl Assign {
    pub fn new(
        destination: Expression,
        value: Expression,
        localization: FileElement<bool>,
    ) -> Assign {
        return Assign {
            destination,
            value,
            localization,
        };
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
    fn get_len(&self) -> usize {
        self.localization.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.localization.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.localization.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.localization.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.localization.get_end_char()
    }
}

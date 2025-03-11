use super::{AstNode, Block, Defs};
use crate::{
    common::{
        localizable::Localizable,
        types::{FileElement, Node, Tree},
    },
    parser::Lexem,
};

pub struct Root {
    defs: Defs,
    block: Block,
    location: FileElement<bool>,
}

impl AstNode for Root {}

impl From<Tree<FileElement<Lexem>>> for Root {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let defs: Defs = Defs::from(root.borrow().get_children()[1].clone());

        let block = Block::from(root.clone());

        let location = FileElement {
            start_line: root.borrow().get_value().get_start_line(),
            end_line: root.borrow().get_value().get_end_line(),
            start_char: root.borrow().get_value().get_start_char(),
            len: root.borrow().get_value().len,
            element: true,
        };

        return Root {
            defs,
            block,
            location,
        };
    }
}

impl Into<Tree<String>> for Root {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("ROOT"));

        root.borrow_mut().add_child(&root, self.defs.into());
        root.borrow_mut().add_child(&root, self.block.into());

        return root;
    }
}

impl Localizable for Root {
    fn get_len(&self) -> usize {
        self.location.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.location.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.location.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.location.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.location.get_end_char()
    }
}

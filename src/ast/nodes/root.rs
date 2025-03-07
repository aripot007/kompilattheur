use super::{AstNode, Block, Defs};
use crate::{
    common::{localizable::Localizable, types::{FileElement, Node, Tree}},
    parser::Lexem,
};

pub struct Root {
    pub defs: Defs,
    pub block: Block,
}

impl AstNode for Root {}

impl From<Tree<FileElement<Lexem>>> for Root {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let defs: Defs = Defs::from(root.borrow().get_children()[1].clone());

        return Root {
            defs,
            block: Block::from(root.clone()),
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

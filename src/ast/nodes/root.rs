use crate::{common::types::{FileElement, Node, Tree}, parser::Lexem};
use super::{AstNode, Block, Defs};


pub struct Root {
    defs: Defs,
    block: Block,
}

impl AstNode for Root {}

impl From<Tree<FileElement<Lexem>>> for Root {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let defs: Defs = Defs::from(root.borrow().get_children()[1].clone());
        
        return Root {defs, block: Block::from(root.clone())};
    }
}

impl Into<Tree<String>> for Root {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("ROOT"));

        root.borrow_mut().add_child(self.defs.into());
        root.borrow_mut().add_child(self.block.into());

        return root;
    }
}

use crate::{common::types::{FileElement, Node, Tree}, parser::Lexem};
use super::{parse_list, AstNode, Def};

pub struct Defs {
    defs: Vec<Def>,
}

impl AstNode for Defs {}

impl From<Tree<FileElement<Lexem>>> for Defs {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let defs: Vec<Def> = parse_list(root, Def::from);
        
        return Defs {defs};
    }
}

impl Into<Tree<String>> for Defs {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("DEFS"));

        for def in self.defs {
            root.borrow_mut().add_child(&root, def.into());
        }

        return root;
    }
}

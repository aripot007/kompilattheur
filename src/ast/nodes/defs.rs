use crate::{common::types::{FileElement, Node, Tree}, parser::Lexem};
use super::{list_into_tree, parse_list, AstNode, Def};

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
        list_into_tree!("DEFS", self.defs)
    }
}

use crate::{common::types::{FileElement, Node, Tree}, parser::Lexem};
use super::{parse_list, AstNode};

pub struct Def {

}

impl AstNode for Def {}

impl From<Tree<FileElement<Lexem>>> for Def {
    fn from(_root: Tree<FileElement<Lexem>>) -> Self {

        return Def {};
    }
}

impl Into<Tree<String>> for Def {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("DEF"));

        return root;
    }
}

use crate::{common::types::{FileElement, Node, Tree}, parser::Lexem};

use super::AstNode;


pub struct Block {

}

impl AstNode for Block {}

impl From<Tree<FileElement<Lexem>>> for Block {
    fn from(_root: Tree<FileElement<Lexem>>) -> Self {
        return Block {};
    }
}

impl Into<Tree<String>> for Block {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("BLOCK"));

        return root;
    }
}

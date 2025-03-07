use super::{list_into_tree, parse_list, AstNode, Def};
use crate::{
    common::{localizable::Localizable, types::{FileElement, Node, Tree}},
    parser::Lexem,
};

pub struct Defs {
    pub defs: Vec<Def>,
}

impl AstNode for Defs {}

impl From<Tree<FileElement<Lexem>>> for Defs {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let defs: Vec<Def> = parse_list(root, Def::from);

        return Defs { defs };
    }
}

impl Into<Tree<String>> for Defs {
    fn into(self) -> Tree<String> {
        list_into_tree!("DEFS", self.defs)
    }
}

impl Localizable for Defs {
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

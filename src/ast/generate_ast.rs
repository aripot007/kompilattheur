use crate::{common::types::{FileElement, Tree}, parser::Lexem};

use super::nodes::Root;

pub fn generate_ast(cst: Tree<FileElement<Lexem>>) -> Root {
    return Root::from(cst);    
}
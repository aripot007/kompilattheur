use crate::{common::types::{file_element::file_element_from, FileElement, IdToken, Node, Token, Tree}, parser::Lexem};
use super::{AstNode, Block, Expression};


pub struct For {
    var: FileElement<IdToken>,
    iterator: Expression,
    block: Block,
}

impl AstNode for For {}

impl From<Tree<FileElement<Lexem>>> for For {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let id_elem = root.borrow().get_children()[1].borrow().get_value();
        let id_token = match id_elem.element {
            Lexem::Terminal(Token::Identifier(id_t)) => id_t,
            t => panic!("Unexpected child #1 of FOR node : expected IdToken, got {}", t),
        };
        let var: FileElement<IdToken> = file_element_from!(id_elem, id_token);
        
        let iterator = Expression::from(root.borrow().get_children()[3].clone());

        let block = Block::from(root.borrow().get_children()[5].clone());

        return For {var, iterator, block};
    }
}

impl Into<Tree<String>> for For {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("FOR"));

        root.borrow_mut().add_child(Node::new(format!("Identifier {}", self.var.element.id)));

        root.borrow_mut().add_child(self.iterator.into());

        root.borrow_mut().add_child(self.block.into());

        return root;
    }
}

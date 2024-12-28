use crate::{common::types::{FileElement, Node, Token, Tree}, parser::Lexem};
use super::{AstNode, Block, Expression};


pub struct Conditional {
    condition: Expression,
    if_block: Block,
    else_block: Option<Block>,
}

impl AstNode for Conditional {}

impl From<Tree<FileElement<Lexem>>> for Conditional {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let if_elem = root.borrow().get_children()[0].borrow().get_value();
        match if_elem.element {
            Lexem::Terminal(Token::If) => (),
            t => panic!("Unexpected child #1 of COND node : expected If, got {}", t),
        };

        let condition = Expression::from(root.borrow().get_children()[1].clone());

        let if_block = Block::from(root.borrow().get_children()[3].clone());

        let else_block = parse_else(root.borrow().get_children()[4].clone());

        return Conditional {condition, if_block, else_block};
    }
}

fn parse_else(else_root: Tree<FileElement<Lexem>>) -> Option<Block> {

    if else_root.borrow().get_children().len() == 0 {
        return None;
    }
    return Some(Block::from(else_root.borrow().get_children()[2].clone()));
}

impl Into<Tree<String>> for Conditional {
    fn into(self) -> Tree<String> {

        let root = match self.else_block.is_some() {
            true => Node::new(String::from("IF-ELSE")),
            false => Node::new(String::from("IF")),
        };

        root.borrow_mut().add_child(&root, self.condition.into());

        root.borrow_mut().add_child(&root, self.if_block.into());

        if let Some(else_block) = self.else_block {
            root.borrow_mut().add_child(&root, else_block.into());
        }

        return root;
    }
}

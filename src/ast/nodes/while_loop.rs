use super::{Ast, AstNode, Block, Expression};
use crate::{
    common::{
        localizable::Localizable,
        types::{FileElement, Node, Tree},
    },
    parser::Lexem,
};

pub struct While {
    pub condition: Expression,
    pub block: Block,
    pub localization: FileElement<bool>,
}

impl AstNode for While {
    fn get_string_repr(&self) -> String {
        String::from("While")
    }
}

impl From<Ast> for While {
    fn from(value: Ast) -> Self {
        match value {
            Ast::While(s) => s,
            _ => panic!("Cannot convert {} to For", value),
        }
    }
}

impl From<Tree<FileElement<Lexem>>> for While {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let condition = Expression::from(root.borrow().get_children()[1].clone());

        let block = Block::from(root.borrow().get_children()[3].clone());

        let localization = FileElement {
            element: true,
            len: root.borrow().get_children()[0].borrow().get_value().len,
            start_char: root.borrow().get_children()[0]
                .borrow()
                .get_value()
                .start_char,
            start_line: root.borrow().get_children()[0]
                .borrow()
                .get_value()
                .start_line,
            end_line: root.borrow().get_children()[0]
                .borrow()
                .get_value()
                .end_line,
        };

        return While {
            condition,
            block,
            localization,
        };
    }
}

impl Into<Tree<String>> for While {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("WHILE"));

        root.borrow_mut().add_child(&root, self.condition.into());

        root.borrow_mut().add_child(&root, self.block.into());

        return root;
    }
}

impl Localizable for &While {
    fn get_len(&self) -> usize {
        self.localization.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.localization.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.localization.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.localization.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.localization.get_end_char()
    }
}

impl Localizable for While {
    fn get_len(&self) -> usize {
        self.localization.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.localization.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.localization.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.localization.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.localization.get_end_char()
    }
}

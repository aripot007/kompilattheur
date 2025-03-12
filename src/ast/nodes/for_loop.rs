use super::{AstNode, Block, Expression};
use crate::{
    common::{
        localizable::Localizable,
        types::{file_element::file_element_from, FileElement, IdToken, Node, Token, Tree},
    },
    parser::Lexem,
};

pub struct For {
    var: FileElement<IdToken>,
    iterator: Expression,
    block: Block,
    pub localization: FileElement<bool>,
}

impl AstNode for For {}

impl From<Tree<FileElement<Lexem>>> for For {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let id_elem = root.borrow().get_children()[1].borrow().get_value();
        let id_token = match id_elem.element {
            Lexem::Terminal(Token::Identifier(id_t)) => id_t,
            t => panic!(
                "Unexpected child #1 of FOR node : expected IdToken, got {}",
                t
            ),
        };
        let var: FileElement<IdToken> = file_element_from!(id_elem, id_token);

        let iterator = Expression::from(root.borrow().get_children()[3].clone());

        let block = Block::from(root.borrow().get_children()[5].clone());

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

        return For {
            var,
            iterator,
            block,
            localization,
        };
    }
}

impl Into<Tree<String>> for For {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("FOR"));

        root.borrow_mut().add_child(
            &root,
            Node::new(format!("Identifier {}", self.var.element.name)),
        );

        root.borrow_mut().add_child(&root, self.iterator.into());

        root.borrow_mut().add_child(&root, self.block.into());

        return root;
    }
}

impl Localizable for For {
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

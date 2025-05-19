use super::{Ast, AstNode};
use crate::{
    common::{
        localizable::Localizable,
        types::{file_element::file_element_from, FileElement, IdToken, Node, Token, Tree},
    },
    parser::Lexem,
};

pub struct Param {
    pub identifier: FileElement<IdToken>,
}

impl AstNode for Param {
    fn get_string_repr(&self) -> String {
        String::from("Param")
    }
}

impl From<Ast> for Param {
    fn from(value: Ast) -> Self {
        match value {
            Ast::Param(s) => s,
            _ => panic!("Cannot convert {} to Param", value),
        }
    }
}

impl From<Tree<FileElement<Lexem>>> for Param {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let id_elem = root.borrow().get_value();
        let id_token = match id_elem.element {
            Lexem::Terminal(Token::Identifier(id_t)) => id_t,
            t => panic!(
                "Unexpected root for param node : expected IdToken, got {}",
                t
            ),
        };
        let identifier: FileElement<IdToken> = file_element_from!(id_elem, id_token);

        return Param { identifier };
    }
}

impl Into<Tree<String>> for Param {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("PARAM"));

        root.borrow_mut().add_child(
            &root,
            Node::new(format!("Identifier {}", self.identifier.element.name)),
        );

        return root;
    }
}

impl Localizable for Param {
    fn get_len(&self) -> usize {
        self.identifier.get_len()
    }

    fn get_start_line(&self) -> usize {
        self.identifier.get_start_line()
    }

    fn get_end_line(&self) -> usize {
        self.identifier.get_end_line()
    }

    fn get_start_char(&self) -> usize {
        self.identifier.get_start_char()
    }

    fn get_end_char(&self) -> usize {
        self.identifier.get_end_char()
    }
}

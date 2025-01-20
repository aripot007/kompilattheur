use crate::{analysis_table::NonTerminal, ast::nodes::parse_list_filter, common::types::{file_element::file_element_from, FileElement, IdToken, Node, Token, Tree}, parser::Lexem};
use super::{AstNode, Block, Param};

pub struct Def {
    identifier: FileElement<IdToken>,
    params: Vec<Param>,
    block: Block,
}

impl AstNode for Def {}

impl From<Tree<FileElement<Lexem>>> for Def {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let id_elem = root.borrow().get_children()[1].borrow().get_value();
        let id_token = match id_elem.element {
            Lexem::Terminal(Token::Identifier(id_t)) => id_t,
            t => panic!("Unexpected child #1 of <def> node : expected IdToken, got {}", t),
        };
        let identifier: FileElement<IdToken> = file_element_from!(id_elem, id_token);

        fn is_param_node(node: Tree<FileElement<Lexem>>) -> bool {
            match node.borrow().get_value().element {
                Lexem::NonTerminal(NonTerminal::IdentList)
                | Lexem::NonTerminal(NonTerminal::IdentListIdent) => true,
                _ => false,
            }
        }

        let params: Vec<Param> = parse_list_filter(
            root.borrow().get_children()[3].clone(),
            Param::from,
            is_param_node,
        );

        let block = Block::from(root.borrow().get_children()[6].clone());

        return Def {
            identifier,
            params,
            block,
        };
    }
}

impl Into<Tree<String>> for Def {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("DEF"));

        root.borrow_mut().add_child(&root, Node::new(format!("Identifier {}", self.identifier.element.name)));


        let params_root = Node::new(String::from("PARAMS"));

        for param in self.params {
            params_root.borrow_mut().add_child(&root, param.into());
        }

        root.borrow_mut().add_child(&root, params_root);

        root.borrow_mut().add_child(&root, self.block.into());

        return root;
    }
}

use super::Factor;
use super::{AstNode, Expression};
use crate::{
    analysis_table::{get_analysis_table, NonTerminal},
    common::types::{file_element::file_element_from, FileElement, IdToken, Node, Token, Tree},
    parser::Lexem,
};

pub enum Memory {
    Identifier(FileElement<IdToken>),
    Access(Expression, Expression),
    NotImplemented,
}

impl AstNode for Memory {}

impl From<Tree<FileElement<Lexem>>> for Memory {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        // Simple statement

        let identifier_node = root.borrow().get_children()[0].clone();
        let identifier_value = identifier_node.borrow().clone().get_value();
        let id_identifier = match identifier_value.element {
            Lexem::Terminal(Token::Identifier(id_t)) => id_t,
            t => panic!(
                "Unexpected child #1 of MEMORY node : expected IdToken, got {}",
                t
            ),
        };
        let identifier = file_element_from!(identifier_value, id_identifier);

        let simple_stmt_ident = root.borrow().get_children()[1].clone();
        let simple_stmt_ident_childrens = simple_stmt_ident.borrow().get_children();

        let right_simple_stmt_ident =
            simple_stmt_ident_childrens[simple_stmt_ident_childrens.len() - 1].clone();
        let right_simple_stmt_ident_elem = right_simple_stmt_ident.borrow().get_value().element;

        let memory = match right_simple_stmt_ident_elem {
            Lexem::NonTerminal(id) => {
                let node_non_terminal = get_analysis_table().get_non_terminal(id);
                match node_non_terminal {
                    NonTerminal::Expr => Memory::Identifier(identifier),
                    NonTerminal::SimpleStmtExpr => {
                        let expr = Expression::from(
                            right_simple_stmt_ident.borrow().get_children()[1].clone(),
                        );
                        Memory::Access(Expression::Factor(Factor::from(identifier_node)), expr)
                    }
                    _ => panic!("Unexpected non terminal in simple statement identifier"),
                }
            }
            _ => panic!("Unexpected terminal in simple statement identifier"),
        };

        return memory;
    }
}

impl Into<Tree<String>> for Memory {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("MEMORY"));
        return root;
    }
}

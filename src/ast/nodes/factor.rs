use crate::analysis_table::NonTerminal;
use crate::{
    analysis_table::get_analysis_table,
    common::types::{file_element::file_element_from, FileElement, Node, NumToken, Token, Tree},
    parser::Lexem,
};

use super::AstNode;

pub enum Factor {
    Integer(FileElement<u64>),
    String(FileElement<String>),
    True(FileElement<Token>),
    False(FileElement<Token>),
    None(FileElement<Token>),
    Symbol(FileElement<Token>),
}

impl AstNode for Factor {}

impl From<Tree<FileElement<Lexem>>> for Factor {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let table = get_analysis_table();

        let non_terminal: &NonTerminal = match root.borrow().get_value().element {
            Lexem::NonTerminal(id) => table.get_non_terminal(id),
            Lexem::Terminal(Token::Identifier(id)) => {
                return Factor::Symbol(file_element_from!(
                    root.borrow().get_value(),
                    Token::Identifier(id)
                ))
            }
            Lexem::Terminal(token) => panic!("Cannot create factor from terminal '{}'", token),
        };

        match non_terminal {
            NonTerminal::Const => {
                let val = root.borrow().get_children()[0].borrow().get_value();
                match val.element {
                    Lexem::Terminal(Token::Integer(NumToken { value })) => {
                        return Factor::Integer(file_element_from!(val, value))
                    }
                    Lexem::Terminal(Token::String(string)) => {
                        return Factor::String(file_element_from!(val, string))
                    }
                    Lexem::Terminal(Token::True) => {
                        return Factor::True(file_element_from!(val, Token::True))
                    }
                    Lexem::Terminal(Token::False) => {
                        return Factor::False(file_element_from!(val, Token::False))
                    }
                    Lexem::Terminal(Token::None) => {
                        return Factor::None(file_element_from!(val, Token::None))
                    }
                    Lexem::Terminal(token) => panic!(
                        "Malformed CST: Expected NumToken while parsing const, found {}",
                        token
                    ),
                    Lexem::NonTerminal(id) => panic!(
                        "Malformed CST: Expected NumToken while parsing const, found {}",
                        table.get_non_terminal_name(id)
                    ),
                }
            }
            _ => panic!("Cannot create factor from '{}' node", non_terminal),
        };
    }
}

impl Into<Tree<String>> for Factor {
    fn into(self) -> Tree<String> {
        (&self).into()
    }
}

impl Into<Tree<String>> for &Factor {
    fn into(self) -> Tree<String> {
        let s = match self {
            Factor::Integer(file_element) => format!("{}", file_element.element),
            Factor::String(file_element) => format!("{}", file_element.element.escape_debug()),
            Factor::True(_file_element) => String::from("True"),
            Factor::False(_file_element) => String::from("False"),
            Factor::Symbol(_file_element) => String::from("Symbol"),
            Factor::None(_file_element) => String::from("None"),
        };

        let root = Node::new(s);

        return root;
    }
}

use crate::{analysis_table::{self, get_analysis_table, NonTerminal}, common::types::{file_element, FileElement, Node, Tree}, parser::Lexem};

use super::{parse_list, AstNode, Statement};


pub struct Block {
    statements: Vec<Statement>,
}

impl AstNode for Block {}

impl From<Tree<FileElement<Lexem>>> for Block {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
    
        let analysis_table = get_analysis_table();

        let root_non_terminal = match root.borrow().get_value().element {
            Lexem::NonTerminal(id) => analysis_table.get_non_terminal(id),
            _ => panic!("Trying to parse BLOCK from terminal concrete node"),
        };

        let statement_list_root: Tree<FileElement<Lexem>> = Node::new(
            FileElement {len: 0, line: 0, start_char: 0, element: Lexem::NonTerminal(0)}
        );

        // Différencie entre le bloc global, qui part d'un noeud <file>, et 
        // un bloc allieurs dans le programme, qui part d'un noeud <suite>
        if root_non_terminal == &NonTerminal::File {

            // Bloc global

            // Premier statement
            statement_list_root.borrow_mut().add_child(root.borrow().get_children()[2].clone());
            // Suite du bloc
            statement_list_root.borrow_mut().add_child(root.borrow().get_children()[3].clone());

        } else if root_non_terminal == &NonTerminal::Suite {

            // Bloc générique

            // Premier statement
            statement_list_root.borrow_mut().add_child(root.borrow().get_children()[2].clone());
            // Suite du bloc
            statement_list_root.borrow_mut().add_child(root.borrow().get_children()[3].clone());

        } else {
            panic!("Invalid NonTerminal for BLOCK root : expected File or Suite, got {}", root_non_terminal);
        }

        let statements: Vec<Statement> = parse_list(statement_list_root, Statement::from);

        return Block {statements};
    }
}

impl Into<Tree<String>> for Block {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("BLOCK"));

        for stmt in self.statements {
            root.borrow_mut().add_child(stmt.into());
        }

        return root;
    }
}

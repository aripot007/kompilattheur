use crate::{
    analysis_table::NonTerminal,
    common::{
        localizable::Localizable, symbol_table::SymbolTableRef, types::{FileElement, Node, Tree}
    },
    parser::Lexem,
};

use super::{parse_list, AstNode, Statement};

pub struct Block {
    pub statements: Vec<Statement>,
    pub localization: FileElement<bool>,
    pub symbol_table: Option<SymbolTableRef>,
}

impl AstNode for Block {}

impl From<Tree<FileElement<Lexem>>> for Block {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let root_non_terminal = match root.borrow().get_value().element {
            Lexem::NonTerminal(nt) => nt,
            _ => panic!("Trying to parse BLOCK from terminal concrete node"),
        };

        let statement_list_root: Tree<FileElement<Lexem>> = Node::new(FileElement {
            len: 0,
            start_line: 0,
            end_line: 0,
            start_char: 0,
            element: Lexem::NonTerminal(NonTerminal::File),
        });

        // Différencie entre le bloc global, qui part d'un noeud <file>, et
        // un bloc allieurs dans le programme, qui part d'un noeud <suite>
        if root_non_terminal == NonTerminal::File {
            // Bloc global

            // Premier statement
            statement_list_root
                .borrow_mut()
                .add_child(&root, root.borrow().get_children()[2].clone());
            // Suite du bloc
            statement_list_root
                .borrow_mut()
                .add_child(&root, root.borrow().get_children()[3].clone());
        } else if root_non_terminal == NonTerminal::Suite {
            // Bloc générique

            // Premier statement
            statement_list_root
                .borrow_mut()
                .add_child(&root, root.borrow().get_children()[2].clone());
            // Suite du bloc
            statement_list_root
                .borrow_mut()
                .add_child(&root, root.borrow().get_children()[3].clone());
        } else {
            panic!(
                "Invalid NonTerminal for BLOCK root : expected File or Suite, got {}",
                root_non_terminal
            );
        }

        let statements: Vec<Statement> = parse_list(statement_list_root, Statement::from);

        let localization = FileElement {
            element: true,
            len: statements.iter().map(|s| s.get_len()).sum(),
            start_char: statements.first().map_or(0, |s| s.get_start_char()),
            start_line: statements.first().map_or(0, |s| s.get_start_line()),
            end_line: statements.last().map_or(0, |s| s.get_end_line()),
        };

        return Block {
            statements,
            localization,
            symbol_table: None,
        };
    }
}

impl Into<Tree<String>> for Block {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("BLOCK"));

        for stmt in self.statements {
            root.borrow_mut().add_child(&root, stmt.into());
        }

        return root;
    }
}

impl Localizable for Block {
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

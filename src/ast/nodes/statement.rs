use crate::{analysis_table::{get_analysis_table, NonTerminal}, common::types::{FileElement, Node, Token, Tree}, parser::Lexem};

use super::{AstNode, Conditional, Expression, For};


pub enum Statement {
    Print(Expression),
    Return(Expression),
    For(For),
    Conditional(Conditional),
    NotImplemented,
}

impl AstNode for Statement {}

impl From<Tree<FileElement<Lexem>>> for Statement {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let analysis_table = get_analysis_table();

        let root_non_terminal = match root.borrow().get_value().element {
            Lexem::NonTerminal(id) => analysis_table.get_non_terminal(id),
            _ => panic!("Trying to parse STATEMENT from terminal concrete node"),
        };

        if root_non_terminal != &NonTerminal::Stmt {
            panic!("Cannot parse Statement from {} node", root_non_terminal);
        }

        let left_child_elem = root.borrow().get_children()[0].clone().borrow().get_value().element;

        match left_child_elem {
            Lexem::NonTerminal(id) if analysis_table.get_non_terminal(id) == &NonTerminal::SimpleStmt => return parse_simple(root.borrow().get_children()[0].clone()),
            Lexem::Terminal(Token::For) => return Statement::For(For::from(root)),
            Lexem::Terminal(Token::If) => return  Statement::Conditional(Conditional::from(root)),
            _ => return Statement::NotImplemented,
        }
    }
}

fn parse_simple(root: Tree<FileElement<Lexem>>) -> Statement {

    let left_child_elem = root.borrow().get_children()[0].clone().borrow().get_value().element;

    match left_child_elem {
        Lexem::Terminal(Token::Print) => Statement::Print(Expression::from(root.borrow().get_children()[2].clone())),
        Lexem::Terminal(Token::Return) => Statement::Return(Expression::from(root.borrow().get_children()[1].clone())),
        _ => return Statement::NotImplemented,
    }
    
}

impl Into<Tree<String>> for Statement {
    fn into(self) -> Tree<String> {
        match self {
            Statement::NotImplemented => return Node::new(String::from("STMT (NI)")),
            Statement::Print(expr) => {
                let r = Node::new(String::from("PRINT"));
                r.borrow_mut().add_child(expr.into());
                return r
            },
            Statement::Return(expr) => {
                let r = Node::new(String::from("RETURN"));
                r.borrow_mut().add_child(expr.into());
                return r
            },
            Statement::For(for_loop) => return for_loop.into(),
            Statement::Conditional(cdt) => return cdt.into(),

        };
    }
}

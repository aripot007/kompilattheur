use crate::{
    analysis_table::NonTerminal,
    common::types::{file_element::file_element_from, FileElement, Node, Token, Tree},
    parser::Lexem,
};

use super::{Assign, AstNode, Conditional, Expression, For};

pub enum Statement {
    Print(Expression),
    Return(Expression),
    For(For),
    Conditional(Conditional),
    Assign(Assign),
    NotImplemented,
}

impl AstNode for Statement {}

impl From<Tree<FileElement<Lexem>>> for Statement {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let root_non_terminal = match root.borrow().get_value().element {
            Lexem::NonTerminal(nt) => nt,
            _ => panic!("Trying to parse STATEMENT from terminal concrete node"),
        };

        if root_non_terminal != NonTerminal::Stmt {
            panic!("Cannot parse Statement from {} node", root_non_terminal);
        }

        let left_child_elem = root.borrow().get_children()[0]
            .clone()
            .borrow()
            .get_value()
            .element;

        match left_child_elem {
            Lexem::NonTerminal(nt)
                if nt == NonTerminal::SimpleStmt =>
            {
                return parse_simple(root.borrow().get_children()[0].clone())
            }
            Lexem::Terminal(Token::For) => return Statement::For(For::from(root)),
            Lexem::Terminal(Token::If) => return Statement::Conditional(Conditional::from(root)),
            _ => return Statement::NotImplemented,
        }
    }
}

fn parse_simple(root: Tree<FileElement<Lexem>>) -> Statement {
    let left_child_elem = root.borrow().get_children()[0]
        .clone()
        .borrow()
        .get_value()
        .element;

    match left_child_elem {
        Lexem::Terminal(Token::Print) => {
            Statement::Print(Expression::from(root.borrow().get_children()[2].clone()))
        }
        Lexem::Terminal(Token::Return) => {
            Statement::Return(Expression::from(root.borrow().get_children()[1].clone()))
        }
        Lexem::Terminal(Token::Identifier(_)) => parse_ident_stmt(root),
        _ => return Statement::NotImplemented,
    }
}

/// Parse a statement beginning with an identifier token.
/// Can be either an assignment, a function call or a noop
fn parse_ident_stmt(root: Tree<FileElement<Lexem>>) -> Statement {

    let identifier_lexem: FileElement<Lexem> = root.borrow().get_children()[0].borrow().get_value();
    let identifier = file_element_from!(identifier_lexem, identifier_lexem.element);

    let right_child = &root.borrow().get_children()[1];

    if let Lexem::Terminal(Token::Assign) = right_child.borrow().get_children()[0].borrow().get_value().element {
        // Simple assignment
        return Statement::Assign(Assign::from(root.clone()));
    }

    let right_child_children = right_child.borrow().get_children();

    if right_child_children.len() == 3 {

        let simple_stmt_expr_node_childre = right_child_children[2].borrow().get_children();

        if simple_stmt_expr_node_childre.len() == 6 {
            // Complex assignment
            return Statement::Assign(Assign::from(root));
        }
    }

    return Statement::NotImplemented;
}

impl Into<Tree<String>> for Statement {
    fn into(self) -> Tree<String> {
        match self {
            Statement::NotImplemented => return Node::new(String::from("STMT (NI)")),
            Statement::Print(expr) => {
                let r = Node::new(String::from("PRINT"));
                r.borrow_mut().add_child(&r, expr.into());
                return r;
            }
            Statement::Return(expr) => {
                let r = Node::new(String::from("RETURN"));
                r.borrow_mut().add_child(&r, expr.into());
                return r;
            }
            Statement::For(for_loop) => return for_loop.into(),
            Statement::Conditional(cdt) => return cdt.into(),
            Statement::Assign(assign) => return assign.into(),
        };
    }
}

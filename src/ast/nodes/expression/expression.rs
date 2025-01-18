
use crate::{
    analysis_table::{get_analysis_table, NonTerminal}, common::types::{FileElement, Node, Token, Tree}, parser::Lexem
};

use super::{super::{AstNode, Factor}, BinOp, UnOp};

pub enum Expression {
    BINOP(Box<Expression>, BinOp, Box<Expression>),
    UNOP(UnOp, Box<Expression>),
    Factor(Factor),
    NotImplemented,
}

impl AstNode for Expression {}

impl From<Tree<FileElement<Lexem>>> for Expression {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        let analysis_table = get_analysis_table();

        let root_elem = root.borrow().get_value().element;
        let root_elem = match root_elem {
            Lexem::Terminal(Token::Identifier(_)) => return Expression::Factor(Factor::from(root)),
            Lexem::Terminal(token) => panic!("Cannot convert terminal {token} to EXPR"),
            Lexem::NonTerminal(id) => analysis_table.get_non_terminal(id),
        };

        // Check if expression is access :
        if root_elem == &NonTerminal::ExprAccess {
            return parse_access_or_factor(root);
        }

        match root_elem {
            &NonTerminal::ExprNeg | &NonTerminal::ExprNegNoAccess | &NonTerminal::ExprNegNoIdentNoAccess
            | &NonTerminal::ExprNot | &NonTerminal::ExprNotNoAccess | &NonTerminal::ExprNotNoIdentNoAccess
            => {
                let children = root.borrow().get_children();
                if children.len() == 2 {
                    let op: UnOp = match children[0].borrow().get_value().element {
                        Lexem::Terminal(token) => UnOp::from(token),
                        Lexem::NonTerminal(id) => panic!("Cannot convert non terminal node {id} to unary operator"),
                    };
                    return Expression::UNOP(op, Box::from(Expression::from(children[1].clone())));
                }
                return Expression::from(children[0].clone());
            },
            &NonTerminal::ExprAccess => return parse_access_or_factor(root),
            _ => return parse_binop_chain(root),
        }
    }
}

fn parse_binop_chain(root: Tree<FileElement<Lexem>>) -> Expression {

    let leftmost_expr: Expression = Expression::from(root.borrow().get_children()[0].clone());

    fn parse(node: Tree<FileElement<Lexem>>, left_expr: Expression) -> Expression {

        let children = node.borrow().get_children();

        if children.len() == 0 {
            return left_expr;
        } else {
            let op = match children[0].borrow().get_value().element {
                Lexem::Terminal(token) => BinOp::from(token),
                Lexem::NonTerminal(id) => panic!("Cannot convert non terminal {id} to binary operation"),
            };
            let expr = Expression::from(children[1].clone());
            return parse(children[2].clone(), Expression::BINOP(Box::from(left_expr), op, Box::from(expr)));
        }
    }

    return parse(root.borrow().get_children()[1].clone(), leftmost_expr);
}

fn parse_access_or_factor(root: Tree<FileElement<Lexem>>) -> Expression {
    let factor: Expression = Expression::Factor(Factor::from(root.borrow().get_children()[0].clone()));

    fn parse(node: Tree<FileElement<Lexem>>, left_expr: Expression) -> Expression {
        let children = node.borrow().get_children();

        if children.len() == 0 {
            return left_expr;
        } else {
            let expr = Expression::from(children[1].clone());
            return parse(children[3].clone(), Expression::BINOP(Box::from(left_expr), BinOp::ACCESS, Box::from(expr)));
        }
    }

    return parse(root.borrow().get_children()[1].clone(), factor);
}

impl Into<Tree<String>> for Expression {
    fn into(self) -> Tree<String> {
        (&self).into()
    }
}

impl Into<Tree<String>> for &Expression {
    fn into(self) -> Tree<String> {
        match self {
            Expression::BINOP(e1, bin_op, e2) => {
                let root = Node::new((*bin_op).into());
                root.borrow_mut().add_child(&root, (*e1).as_ref().into());
                root.borrow_mut().add_child(&root, (*e2).as_ref().into());
                return root;
            },
            Expression::UNOP(un_op, expression) => {
                let root = Node::new((*un_op).into());
                root.borrow_mut().add_child(&root, (*expression).as_ref().into());
                return root;
            },
            Expression::Factor(f) => f.into(),
            Expression::NotImplemented => Node::new(String::from("EXPR (NI)")),
        }
    }
}

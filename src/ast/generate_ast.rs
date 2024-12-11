use std::{cell::RefCell, rc::Rc};

use crate::{
    common::types::{Node, Token},
    parser::Lexem,
};

/*
TODO:
Créer superset de Lexem pour AST
Transitionner type de l'arbre
Modifier les noeuds de l'arbre avec les nouveaux types
*/

pub fn generate_ast(tree: Rc<RefCell<Node<Lexem>>>) {
    remove_syntax_terminals(tree.clone());
    remove_empty_non_terminals(tree.clone()).unwrap_or(Node::new(Lexem::NonTerminal(0)));
    lift_single_child_nodes(tree.clone());
    simplify_terminal_nodes(tree);
}

fn simplify_terminal_nodes(node: Rc<RefCell<Node<Lexem>>>) {
    let children = node.borrow().get_children().clone();
    if children.len() == 1 && !is_non_terminal(&children[0]) {
        let terminal_value = children[0].borrow().value.clone();
        node.borrow_mut().value = terminal_value;
        node.borrow_mut().set_children(&node, vec![]);
    } else {
        for child in children {
            simplify_terminal_nodes(child);
        }
    }
}

fn remove_empty_non_terminals(node: Rc<RefCell<Node<Lexem>>>) -> Option<Rc<RefCell<Node<Lexem>>>> {
    if is_non_terminal(&node) && node.borrow().get_children().is_empty() {
        return None;
    }
    let children = node.borrow().get_children().clone();
    let mut new_children = vec![];
    for child in children {
        if let Some(non_empty_child) = remove_empty_non_terminals(child.clone()) {
            new_children.push(non_empty_child);
        }
    }
    node.borrow_mut().set_children(&node, new_children);
    Some(node)
}

fn is_non_terminal(node: &Rc<RefCell<Node<Lexem>>>) -> bool {
    match node.borrow().value {
        Lexem::NonTerminal(_) => true,
        _ => false,
    }
}

fn lift_single_child_nodes(node: Rc<RefCell<Node<Lexem>>>) {
    let mut children = node.borrow().get_children().clone();
    while children.len() == 1 && is_non_terminal(&children[0]) {
        let grand_children = children[0].borrow().get_children().clone();
        children = grand_children;
    }
    node.borrow_mut().set_children(&node, children.clone());
    for child in children {
        lift_single_child_nodes(child);
    }
}

fn remove_syntax_terminals(node: Rc<RefCell<Node<Lexem>>>) {
    let children = node.borrow().get_children().clone();
    let mut new_children = vec![];
    for child in children {
        if !is_syntax_terminal(child.clone()) {
            new_children.push(child);
        }
    }
    node.borrow_mut().set_children(&node, new_children.clone());
    for child in new_children {
        remove_syntax_terminals(child);
    }
}

fn is_syntax_terminal(node: Rc<RefCell<Node<Lexem>>>) -> bool {
    match node.borrow().value {
        Lexem::Terminal(ref t) => match t {
            Token::Begin
            | Token::CloseBracket
            | Token::CloseParenthesis
            | Token::Comma
            | Token::Def
            | Token::EOF
            | Token::End
            | Token::Newline
            | Token::OpenBracket
            | Token::OpenParenthesis
            | Token::Sep => true,
            _ => false,
        },
        _ => false,
    }
}


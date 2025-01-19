use super::AstNode;
use crate::{
    analysis_table::NonTerminal, ast::nodes::parse_access, common::types::{file_element::{empty_file_elt, file_element_from}, FileElement, Node, Tree}, parser::Lexem
};

use super::Expression;

pub struct Assign {
    destination: Expression,
    value: Expression,
}

impl AstNode for Assign {}

impl From<Tree<FileElement<Lexem>>> for Assign {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {

        /* We need to reconstruct the left expression tree

        We move the nodes to obtain a tree that looks like that :

                     expr
                    /    \
            access_expr   or_expr ...
                /       \
            factor      access_expr_suite ...
            /     \
            id    factor_ident

        The tree can then be parsed by Expression::from
        */

        let right_child = root.borrow().get_children()[1].clone();

        let (destination, value): (Expression, Expression) =
            match right_child.borrow().get_value().element {
                // Real assignation to identifier
                Lexem::NonTerminal(NonTerminal::SimpleStateIdent) => {

                    let simple_state_ident_child2_elt = right_child.borrow().get_children()[1].borrow().get_value().element;
                    
                    if let Lexem::NonTerminal(NonTerminal::Expr) = simple_state_ident_child2_elt {
                        // Simple id = expr assignment
                        parse_simple_id_assign_tree(root)
                    } else {
                        // Complex expr1 = expr2 with identifier starting expr1
                        parse_complex_id_assign_tree(root)
                    }
                }
                _ => todo!()
        };

        return Assign { destination, value };
    }
}

/// Descend into childrens with the given indexes
macro_rules! descend_children {
    ($root: expr, $n: expr) => {
        $root.borrow().get_children()[$n]
    };

    ($root: expr, $n: expr, $($children:expr),+) => {
        descend_children!(descend_children!($root, $n), $($children),+)
    }
}

/// Add a non terminal child with an empty fileelement to the given root
macro_rules! add_nonterm_child {
    ($root: expr, $nt: expr) => {
        $root.borrow_mut().add_child(&$root, Node::new(empty_file_elt!(Lexem::NonTerminal($nt))));
    };
}

/// The first case is a simple id = expr assignement, with the following tree :
///
///             simple_stmt
///               /    \ 
///             id      simple_stmt_ident
///                        /       \
///                       =        expr
///
/// We just need a factor tree in this case, that looks like this :
///
///               factor
///              /      \
///             id      factor_id
fn parse_simple_id_assign_tree(root: Tree<FileElement<Lexem>>) -> (Expression, Expression) {

    let right_expr: Tree<FileElement<Lexem>> = descend_children!(root, 1, 1).clone();
    let ident_node: Tree<FileElement<Lexem>> = descend_children!(root, 0).clone();
    
    let left_expr: Tree<FileElement<Lexem>> = Node::new(file_element_from!(ident_node.borrow().get_value(), Lexem::NonTerminal(NonTerminal::Factor)));

    left_expr.borrow_mut().add_child(&left_expr, ident_node);
    add_nonterm_child!(left_expr, NonTerminal::FactorIdent);

    return (Expression::from(left_expr), Expression::from(right_expr));
}

/// The second case is a expr1 = expr2 assignement, with expr1 starting with an id the following tree :
///
///                         simple_stmt
///                        /          \ 
///                     id          simple_stmt_ident
///                        ________/        |        \______________
///                       /                 |                       \
///                      /                  |                        \
///             factor_id             expr2_no_access                simple_stmt_expr ___________
///                                  /   |        \                 /    \          \___         \
///                                or  expr_and  expr2_no_access   /      \              =S        \
///                                                  ...       [ expr ]    access_suite            expr          
///                                                                         /       \
///                                                                             ...
/// We move create a factor tree for the left identifier, embeded in a normal expr tree to handle the ors etc,
/// and finally move the access exprs from the right to the access part of the rightmost factor in the leftmost expr 
///
///                         expr
///                        /    \______
///                       /            \
///                      /              \
///                   factor        expr2_no_access
///                  /      \           ...
///                id    factor_id  
///   
fn parse_complex_id_assign_tree(root: Tree<FileElement<Lexem>>) -> (Expression, Expression) {
    
    let value_expr: Tree<FileElement<Lexem>> = descend_children!(root, 1, 2, 5).clone();

    let id_node = Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::Factor)));

    // Move identifier
    id_node.borrow_mut().add_child(&id_node, root.borrow().get_children()[0].clone());
    
    // Move factor id
    id_node.borrow_mut().add_child(&id_node, descend_children!(root, 1, 0).clone());

    // Make fake left expr tree, without access
    let left_expr_tree: Tree<FileElement<Lexem>> = Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::Expr)));

    // Add identifier as factor
    left_expr_tree.borrow_mut().add_child(&left_expr_tree, id_node);

    // Add the rest of the expression tree
    left_expr_tree.borrow_mut().add_child(&left_expr_tree, descend_children!(root, 1, 1).clone());

    let left_expr = Expression::from(left_expr_tree);

    // Add the access expression to the rightmost factor

    fn insert_access(expr: Expression, access_root: Tree<FileElement<Lexem>>) -> Expression {
        match expr {
            Expression::BINOP(e1, op, e2) => Expression::BINOP(e1, op, Box::new(insert_access(*e2, access_root))),
            Expression::UNOP(op, e) => Expression::UNOP(op, Box::new(insert_access(*e, access_root))),
            Expression::NotImplemented => Expression::NotImplemented,
            Expression::Factor(_) => parse_access(access_root, expr),
        }
    }

    // Construct fake access tree
    let access_root: Tree<FileElement<Lexem>> = Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::ExprAccess)));

    // Move the first 4 childs of the simple_stmt_expr
    let simple_stmt_expr_children = descend_children!(root, 1, 2).borrow().get_children();
    for i in 0 ..= 3 {
        access_root.borrow_mut().add_child(&access_root, simple_stmt_expr_children[i].clone());
    }

    let left_expr = insert_access(left_expr, access_root);

    return (left_expr, Expression::from(value_expr));
}


impl Into<Tree<String>> for Assign {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("ASSIGN"));

        root.borrow_mut().add_child(&root, self.destination.into());
        root.borrow_mut().add_child(&root, self.value.into());

        return root;
    }
}

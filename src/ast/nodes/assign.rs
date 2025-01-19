use super::AstNode;
use crate::{
    common::types::{FileElement, Node, Tree}, parser::Lexem
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

        /*
        
        let identifier_node = &root.borrow().get_children()[0];
        
        match identifier_node.borrow().get_value().element {
            Lexem::Terminal(Token::Identifier(id)) => (),
            _ => panic!("Unsupported assign from node :\n {}", root.borrow().generate_mermaid())
        }

        

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
        */

        let dest = Expression::from(root.borrow().get_children()[0].clone());

        let value = Expression::from(
            root.borrow() // simple statement
                .get_children()[1]
                .borrow() // simple statement identifier
                .get_children()[1]
                .clone(),
        );

        return Assign { destination: dest, value };
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
fn reconstruct_simple_assign_tree(root: Tree<FileElement<Lexem>>) -> (Tree<FileElement<Lexem>>, Tree<FileElement<Lexem>>) {

    let right_expr: Tree<FileElement<Lexem>> = descend_children!(root, 1, 1).clone();
    let ident_node: Tree<FileElement<Lexem>> = descend_children!(root, 0).clone();
    todo!()
}

impl Into<Tree<String>> for Assign {
    fn into(self) -> Tree<String> {
        let root = Node::new(String::from("ASSIGN"));

        root.borrow_mut().add_child(&root, self.destination.into());
        root.borrow_mut().add_child(&root, self.value.into());

        return root;
    }
}

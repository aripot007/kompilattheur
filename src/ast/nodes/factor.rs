use crate::analysis_table::NonTerminal;
use crate::ast::nodes::parse_list_filter;
use crate::common::types::IdToken;
use crate::{
    common::types::{file_element::file_element_from, FileElement, Node, NumToken, Token, Tree},
    parser::Lexem,
};

use super::{list_into_tree, AstNode, Expression};

pub enum Factor {
    Integer(FileElement<u64>),
    String(FileElement<String>),
    True(FileElement<Token>),
    False(FileElement<Token>),
    None(FileElement<Token>),
    Identifier(IdToken),
    List(Vec<Expression>),
    Expr(Box<Expression>),
    Call {
        identifier: IdToken,
        args: Vec<Expression>,
    },
}

impl AstNode for Factor {}

impl From<Tree<FileElement<Lexem>>> for Factor {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        // Check if a node from an expr list is an element or a separator
        fn is_arg_node(node: Tree<FileElement<Lexem>>) -> bool {
            match node.borrow().get_value().element {
                Lexem::NonTerminal(NonTerminal::ExprList)
                | Lexem::NonTerminal(NonTerminal::ExprListExpr) => true,
                _ => false,
            }
        }

        if let Lexem::Terminal(Token::Identifier(id)) = root.borrow().get_value().element {
            return Factor::Identifier(id);
        }

        let children = root.borrow().get_children();

        // Const
        if children.len() == 1 {
            let val = children[0].borrow().get_children()[0].borrow().get_value();

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
                Lexem::NonTerminal(nt) => panic!(
                    "Malformed CST: Expected NumToken while parsing const, found {}",
                    nt
                ),
            }
        } else if children.len() == 2 {
            // Identifier or function call

            let identifier = match children[0].borrow().get_value().element {
                Lexem::Terminal(Token::Identifier(id)) => id,
                _ => panic!(
                    "Invalid identifier child for node {}",
                    root.borrow().generate_html()
                ),
            };

            let right_child_children = children[1].borrow().get_children();

            if right_child_children.len() == 0 {
                // Identifier only
                return Factor::Identifier(identifier);
            }

            // Function call

            let args: Vec<Expression> = parse_list_filter(
                right_child_children[1].clone(),
                Expression::from,
                is_arg_node,
            );

            return Factor::Call { identifier, args };
        } else if children.len() == 3 {
            // Expr or list
            match children[0].borrow().get_value().element {
                Lexem::Terminal(Token::OpenBracket) => {
                    return Factor::List(parse_list_filter(
                        children[1].clone(),
                        Expression::from,
                        is_arg_node,
                    ))
                }
                Lexem::Terminal(Token::OpenParenthesis) => {
                    return Factor::Expr(Box::new(Expression::from(children[1].clone())))
                }
                _ => panic!(),
            }
        }

        println!("Not recognized : {}\n", root.borrow().generate_mermaid());

        return Factor::Call {
            identifier: IdToken {
                id: 999,
                name: String::from("NI"),
            },
            args: Vec::new(),
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
            Factor::String(file_element) => {
                format!("String : \"{}\"", file_element.element.escape_debug())
            }
            Factor::True(_file_element) => String::from("True"),
            Factor::False(_file_element) => String::from("False"),
            Factor::Identifier(id) => format!("Identifier {}", id.name),
            Factor::None(_file_element) => String::from("None"),
            Factor::List(vec) => return list_into_tree!("LIST", vec),
            Factor::Expr(expression) => return (*expression).as_ref().into(),
            Factor::Call { identifier, args } => {
                let root = Node::new(String::from("CALL"));

                root.borrow_mut()
                    .add_child(&root, Node::new(format!("Identifier {}", identifier.name)));
                root.borrow_mut()
                    .add_child(&root, list_into_tree!("ARGS", args));

                return root;
            }
        };

        let root = Node::new(s);

        return root;
    }
}

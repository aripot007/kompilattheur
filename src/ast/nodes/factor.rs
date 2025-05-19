use crate::analysis_table::NonTerminal;
use crate::ast::nodes::parse_list_filter;
use crate::common::localizable::Localizable;
use crate::common::types::IdToken;
use crate::typing::Type;
use crate::{
    common::types::{file_element::file_element_from, FileElement, Node, NumToken, Token, Tree},
    parser::Lexem,
};

use super::{list_into_tree, AstNode, Expression};

pub struct Factor {
    pub factor_type: Option<Type>,
    pub kind: FactorKind,
}

pub enum FactorKind {
    Integer(FileElement<u64>),
    String(FileElement<String>),
    True(FileElement<Token>),
    False(FileElement<Token>),
    None(FileElement<Token>),
    Identifier(FileElement<IdToken>),
    List(Vec<Expression>),
    Expr(Box<Expression>),
    Call {
        identifier: IdToken,
        args: Vec<Expression>,
        localization: FileElement<bool>,
    },
}

impl AstNode for Factor {
    fn get_string_repr(&self) -> String {
        String::from(match &self.kind {
            FactorKind::Integer(_) => "Factor::Integer",
            FactorKind::String(_) => "Factor::String",
            FactorKind::True(_) => "Factor::True",
            FactorKind::False(_) => "Factor::False",
            FactorKind::None(_) => "Factor::None",
            FactorKind::Identifier(_) => "Factor::Identifier",
            FactorKind::List(_) => "Factor::List",
            FactorKind::Expr(_) => "Factor::Expr",
            FactorKind::Call {
                identifier: _,
                args: _,
                localization: _,
            } => "Factor::Call",
        })
    }
}

impl From<FactorKind> for Factor {
    fn from(value: FactorKind) -> Self {
        Factor {
            factor_type: None,
            kind: value,
        }
    }
}

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
            return FactorKind::Identifier(file_element_from!(root.borrow().get_value(), id))
                .into();
        }

        let children = root.borrow().get_children();

        // Const
        if children.len() == 1 {
            let val = children[0].borrow().get_children()[0].borrow().get_value();

            match val.element {
                Lexem::Terminal(Token::Integer(NumToken { value })) => {
                    return FactorKind::Integer(file_element_from!(val, value)).into()
                }
                Lexem::Terminal(Token::String(string)) => {
                    return FactorKind::String(file_element_from!(val, string)).into()
                }
                Lexem::Terminal(Token::True) => {
                    return FactorKind::True(file_element_from!(val, Token::True)).into()
                }
                Lexem::Terminal(Token::False) => {
                    return FactorKind::False(file_element_from!(val, Token::False)).into()
                }
                Lexem::Terminal(Token::None) => {
                    return FactorKind::None(file_element_from!(val, Token::None)).into()
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
                return FactorKind::Identifier(file_element_from!(
                    children[0].borrow().get_value(),
                    identifier
                ))
                .into();
            }

            // Function call

            let args: Vec<Expression> = parse_list_filter(
                right_child_children[1].clone(),
                Expression::from,
                is_arg_node,
            );

            let identifier_node = children[0].borrow().get_value();
            let localization = FileElement {
                element: true,
                len: identifier_node.get_len() + args.iter().map(|s| s.get_len()).sum::<usize>(),
                start_char: identifier_node.get_start_char(),
                start_line: identifier_node.get_start_line(),
                end_line: args.last().map_or(0, |s| s.get_end_line()),
            };

            return Factor {
                factor_type: None,
                kind: FactorKind::Call {
                    identifier,
                    args,
                    localization,
                },
            };
        } else if children.len() == 3 {
            // Expr or list
            match children[0].borrow().get_value().element {
                Lexem::Terminal(Token::OpenBracket) => {
                    return FactorKind::List(parse_list_filter(
                        children[1].clone(),
                        Expression::from,
                        is_arg_node,
                    ))
                    .into()
                }
                Lexem::Terminal(Token::OpenParenthesis) => {
                    return FactorKind::Expr(Box::new(Expression::from(children[1].clone()))).into()
                }
                _ => panic!(),
            }
        }

        panic!("Not recognized : {}\n", root.borrow().generate_mermaid());
    }
}

impl Into<Tree<String>> for Factor {
    fn into(self) -> Tree<String> {
        (&self).into()
    }
}

impl Into<Tree<String>> for &Factor {
    fn into(self) -> Tree<String> {
        let mut s = match &self.kind {
            FactorKind::Integer(file_element) => format!("{}", file_element.element),
            FactorKind::String(file_element) => {
                format!("String : \"{}\"", file_element.element.escape_debug())
            }
            FactorKind::True(_file_element) => String::from("True"),
            FactorKind::False(_file_element) => String::from("False"),
            FactorKind::Identifier(id) => format!("Identifier {}", id.element.name),
            FactorKind::None(_file_element) => String::from("None"),
            FactorKind::List(vec) => return list_into_tree!("LIST", vec),
            FactorKind::Expr(expr_box) => return expr_box.as_ref().into(),
            FactorKind::Call {
                identifier,
                args,
                localization: _,
            } => {
                // Add typing information if available
                let s = match &self.factor_type {
                    Some(t) => format!("CALL\n({})", t),
                    _ => String::from("CALL"),
                };

                let root = Node::new(s);

                root.borrow_mut()
                    .add_child(&root, Node::new(format!("Identifier {}", identifier.name)));
                root.borrow_mut()
                    .add_child(&root, list_into_tree!("ARGS", args));

                return root;
            }
        };

        // Add typing information if available
        if let Some(t) = &self.factor_type {
            s = format!("{}\n({})", s, t);
        }

        let root = Node::new(s);

        return root;
    }
}

impl Localizable for Factor {
    fn get_start_line(&self) -> usize {
        (&self).get_start_line()
    }

    fn get_end_line(&self) -> usize {
        (&self).get_end_line()
    }

    fn get_start_char(&self) -> usize {
        (&self).get_start_char()
    }

    fn get_end_char(&self) -> usize {
        (&self).get_end_char()
    }

    fn get_len(&self) -> usize {
        (&self).get_len()
    }
}

impl Localizable for &Factor {
    fn get_len(&self) -> usize {
        match &self.kind {
            FactorKind::Integer(fe) => fe.get_len(),
            FactorKind::String(fe) => fe.get_len(),
            FactorKind::True(fe) | FactorKind::False(fe) | FactorKind::None(fe) => fe.get_len(),
            FactorKind::Identifier(fe) => fe.get_len(),
            FactorKind::List(expressions) => expressions.iter().map(|e| e.get_len()).sum(),
            FactorKind::Expr(expression) => expression.get_len(),
            FactorKind::Call {
                identifier: _,
                args: _,
                localization,
            } => localization.get_len(),
        }
    }

    fn get_start_line(&self) -> usize {
        match &self.kind {
            FactorKind::Integer(fe) => fe.get_start_line(),
            FactorKind::String(fe) => fe.get_start_line(),
            FactorKind::True(fe) | FactorKind::False(fe) | FactorKind::None(fe) => {
                fe.get_start_line()
            }
            FactorKind::Identifier(fe) => fe.get_start_line(),
            FactorKind::List(expressions) => match expressions.first() {
                Some(first) => first.get_start_line(),
                None => 1,
            },
            FactorKind::Expr(expression) => expression.get_start_line(),
            FactorKind::Call {
                identifier: _,
                args: _,
                localization,
            } => localization.get_start_line(),
        }
    }

    fn get_end_line(&self) -> usize {
        match &self.kind {
            FactorKind::Integer(fe) => fe.get_end_line(),
            FactorKind::String(fe) => fe.get_end_line(),
            FactorKind::True(fe) | FactorKind::False(fe) | FactorKind::None(fe) => {
                fe.get_end_line()
            }
            FactorKind::Identifier(fe) => fe.get_end_line(),
            FactorKind::List(expressions) => match expressions.last() {
                Some(first) => first.get_end_line(),
                None => 1,
            },
            FactorKind::Expr(expression) => expression.get_end_line(),
            FactorKind::Call {
                identifier: _,
                args: _,
                localization,
            } => localization.get_end_line(),
        }
    }

    fn get_start_char(&self) -> usize {
        match &self.kind {
            FactorKind::Integer(fe) => fe.get_start_char(),
            FactorKind::String(fe) => fe.get_start_char(),
            FactorKind::True(fe) | FactorKind::False(fe) | FactorKind::None(fe) => {
                fe.get_start_char()
            }
            FactorKind::Identifier(fe) => fe.get_start_char(),
            FactorKind::List(expressions) => match expressions.first() {
                Some(first) => first.get_start_char(),
                None => 1,
            },
            FactorKind::Expr(expression) => expression.get_start_char(),
            FactorKind::Call {
                identifier: _,
                args: _,
                localization,
            } => localization.get_start_char(),
        }
    }

    fn get_end_char(&self) -> usize {
        match &self.kind {
            FactorKind::Integer(fe) => fe.get_end_char(),
            FactorKind::String(fe) => fe.get_end_char(),
            FactorKind::True(fe) | FactorKind::False(fe) | FactorKind::None(fe) => {
                fe.get_end_char()
            }
            FactorKind::Identifier(fe) => fe.get_end_char(),
            FactorKind::List(expressions) => match expressions.last() {
                Some(first) => first.get_end_char(),
                None => 1,
            },
            FactorKind::Expr(expression) => expression.get_end_char(),
            FactorKind::Call {
                identifier: _,
                args: _,
                localization,
            } => localization.get_end_char(),
        }
    }
}

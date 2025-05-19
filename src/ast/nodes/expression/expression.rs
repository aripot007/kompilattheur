use crate::{
    analysis_table::NonTerminal,
    ast::nodes::Ast,
    common::{
        localizable::Localizable,
        types::{FileElement, Node, Token, Tree},
    },
    parser::Lexem,
    typing::Type,
};

use super::{
    super::{AstNode, Factor},
    BinOp, UnOp,
};

pub struct Expression {
    pub expr_type: Option<Type>,
    pub kind: ExpressionKind,
}

pub enum ExpressionKind {
    BINOP(Box<Expression>, BinOp, Box<Expression>),
    UNOP(UnOp, Box<Expression>),
    Factor(Factor),
    NotImplemented,
}

impl From<ExpressionKind> for Expression {
    fn from(value: ExpressionKind) -> Self {
        Expression {
            expr_type: None,
            kind: value,
        }
    }
}

impl AstNode for Expression {
    fn get_string_repr(&self) -> String {
        match &self.kind {
            ExpressionKind::BINOP(_, bin_op, _) => {
                format!("Expression::BinOp({})", (*bin_op).to_string())
            }
            ExpressionKind::UNOP(un_op, _) => format!("Expression::UnOp({})", (*un_op).to_string()),
            ExpressionKind::Factor(_) => String::from("Expression::Factor"),
            ExpressionKind::NotImplemented => String::from("Expression::NotImplemented"),
        }
    }
}

impl From<Ast> for Expression {
    fn from(value: Ast) -> Self {
        match value {
            Ast::Expression(s) => s,
            _ => panic!("Cannot convert {} to Expression", value),
        }
    }
}

impl From<Tree<FileElement<Lexem>>> for Expression {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let root_elem = root.borrow().get_value().element;
        let root_elem = match root_elem {
            Lexem::Terminal(Token::Identifier(_)) => {
                return ExpressionKind::Factor(Factor::from(root)).into()
            }
            Lexem::Terminal(token) => panic!("Cannot convert terminal {token} to EXPR"),
            Lexem::NonTerminal(nt) => nt,
        };

        // Check if expression is access :
        if root_elem == NonTerminal::ExprAccess {
            return parse_access_or_factor(root);
        }

        match root_elem {
            NonTerminal::ExprNeg
            | NonTerminal::ExprNegNoAccess
            | NonTerminal::ExprNegNoIdentNoAccess
            | NonTerminal::ExprNot
            | NonTerminal::ExprNotNoAccess
            | NonTerminal::ExprNotNoIdentNoAccess => {
                let children = root.borrow().get_children();
                if children.len() == 2 {
                    let op: UnOp = match children[0].borrow().get_value().element {
                        Lexem::Terminal(token) => UnOp::from(token),
                        Lexem::NonTerminal(id) => {
                            panic!("Cannot convert non terminal node {id} to unary operator")
                        }
                    };
                    return ExpressionKind::UNOP(
                        op,
                        Box::from(Expression::from(children[1].clone())),
                    )
                    .into();
                }
                return Expression::from(children[0].clone());
            }
            NonTerminal::ExprAccess => return parse_access_or_factor(root),
            NonTerminal::Factor | NonTerminal::FactorNoIdent => {
                return ExpressionKind::Factor(Factor::from(root)).into()
            }
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
                Lexem::NonTerminal(id) => {
                    panic!("Cannot convert non terminal {id} to binary operation")
                }
            };
            let expr = Expression::from(children[1].clone());
            return parse(
                children[2].clone(),
                ExpressionKind::BINOP(Box::from(left_expr), op, Box::from(expr)).into(),
            );
        }
    }

    return parse(root.borrow().get_children()[1].clone(), leftmost_expr);
}

/// Parse an access expression associated with a factor
pub(in crate::ast::nodes) fn parse_access(
    access_root: Tree<FileElement<Lexem>>,
    left_expr: Expression,
) -> Expression {
    let children = access_root.borrow().get_children();

    if children.len() == 0 {
        return left_expr;
    } else {
        let expr = Expression::from(children[1].clone());
        return parse_access(
            children[3].clone(),
            ExpressionKind::BINOP(Box::from(left_expr), BinOp::ACCESS, Box::from(expr)).into(),
        );
    }
}

fn parse_access_or_factor(root: Tree<FileElement<Lexem>>) -> Expression {
    let factor: Expression =
        ExpressionKind::Factor(Factor::from(root.borrow().get_children()[0].clone())).into();
    return parse_access(root.borrow().get_children()[1].clone(), factor);
}

impl Into<Tree<String>> for Expression {
    fn into(self) -> Tree<String> {
        (&self).into()
    }
}

impl Into<Tree<String>> for &Expression {
    fn into(self) -> Tree<String> {
        match &self.kind {
            ExpressionKind::BINOP(e1, bin_op, e2) => {
                let s: String = match &self.expr_type {
                    Some(t) => format!("{}\n({})", *bin_op, t),
                    None => (*bin_op).into(),
                };
                let root = Node::new(s);
                root.borrow_mut().add_child(&root, (*e1).as_ref().into());
                root.borrow_mut().add_child(&root, (*e2).as_ref().into());
                return root;
            }
            ExpressionKind::UNOP(un_op, expression) => {
                let s: String = match &self.expr_type {
                    Some(t) => format!("{}\n({})", *un_op, t),
                    None => (*un_op).into(),
                };
                let root = Node::new(s);
                root.borrow_mut()
                    .add_child(&root, (*expression).as_ref().into());
                return root;
            }
            #[cfg(feature = "debug-typing")]
            ExpressionKind::Factor(f) => {
                let mut root = f.into();

                // Add ghost type node for debugging
                if let Some(t) = &self.expr_type {
                    let fake_root = Node::new(format!("(Typed Expr::Factor : {})", t));
                    fake_root.borrow_mut().add_child(&fake_root, root);
                    root = fake_root;
                }

                root
            }
            #[cfg(not(feature = "debug-typing"))]
            ExpressionKind::Factor(f) => f.into(),
            ExpressionKind::NotImplemented => Node::new(String::from("EXPR (NI)")),
        }
    }
}

impl Localizable for Expression {
    fn get_len(&self) -> usize {
        match &self.kind {
            ExpressionKind::BINOP(e1, _bin_op, e2) => e1.get_len() + e2.get_len(),
            ExpressionKind::UNOP(_un_op, expression) => expression.get_len(),
            ExpressionKind::Factor(f) => f.get_len(),
            ExpressionKind::NotImplemented => 0,
        }
    }

    fn get_start_line(&self) -> usize {
        match &self.kind {
            ExpressionKind::BINOP(e1, _bin_op, _e2) => e1.get_start_line(),
            ExpressionKind::UNOP(_un_op, expression) => expression.get_start_line(),
            ExpressionKind::Factor(f) => f.get_start_line(),
            ExpressionKind::NotImplemented => 0,
        }
    }

    fn get_end_line(&self) -> usize {
        match &self.kind {
            ExpressionKind::BINOP(_e1, _bin_op, e2) => e2.get_end_line(),
            ExpressionKind::UNOP(_un_op, expression) => expression.get_end_line(),
            ExpressionKind::Factor(f) => f.get_end_line(),
            ExpressionKind::NotImplemented => 0,
        }
    }

    fn get_start_char(&self) -> usize {
        match &self.kind {
            ExpressionKind::BINOP(e1, _bin_op, _e2) => e1.get_start_char(),
            ExpressionKind::UNOP(_un_op, expression) => expression.get_start_char(),
            ExpressionKind::Factor(f) => f.get_start_char(),
            ExpressionKind::NotImplemented => 0,
        }
    }

    fn get_end_char(&self) -> usize {
        match &self.kind {
            ExpressionKind::BINOP(_e1, _bin_op, e2) => e2.get_end_char(),
            ExpressionKind::UNOP(_un_op, expression) => expression.get_end_char(),
            ExpressionKind::Factor(f) => f.get_end_char(),
            ExpressionKind::NotImplemented => 0,
        }
    }
}

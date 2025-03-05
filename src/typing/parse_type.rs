use crate::ast::nodes::{Ast, BinOp, Expression, Factor, UnOp};

use super::types::Type;

impl TryFrom<Ast> for Type {

    type Error = ();

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        return match value {
            Ast::Expression(expression) => todo!(),
            Ast::Conditional(conditional) => todo!(),
            Ast::Factor(factor) => factor.try_into(),
            Ast::Param(param) => todo!(),
            Ast::Statement(statement) => todo!(),
            Ast::Assign(_)
            | Ast::For(_) => Ok(Type::None),
            Ast::Def(def) => todo!(),
            Ast::Defs(_)
            | Ast::Block(_)
            | Ast::Root(_) => Err(())
        };
    }
}

impl TryFrom<Factor> for Type {
    type Error = ();

    fn try_from(value: Factor) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&Factor> for Type {
    type Error = ();

    fn try_from(factor: &Factor) -> Result<Self, Self::Error> {
        return match factor {
            Factor::Integer(_) => Ok(Type::Int),
            Factor::String(_) => Ok(Type::String),
            Factor::True(_)
            | Factor::False(_) => Ok(Type::Bool),
            Factor::None(_) => Ok(Type::Bool),
            Factor::Identifier(_) => todo!(),
            Factor::List(_) => Ok(Type::List),
            Factor::Expr(expr) => expr.as_ref().try_into(),
            Factor::Call { identifier, args } => todo!(),
        };
    }
}

impl TryFrom<&Expression> for Type {
    type Error = ();

    fn try_from(value: &Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::BINOP(e1, bin_op, e2) => match (e1.as_ref().try_into(), e2.as_ref().try_into()) {
                (Ok(t1), Ok(t2)) if t1 != t2 => Err(try_parse_binop(*bin_op, t1)),
                _ => Err(()),
            },
            Expression::UNOP(un_op, expr) => match (un_op, expr.as_ref().try_into()) {
                (UnOp::NEG, Ok(Type::Int)) => Ok(Type::Int),
                (UnOp::NOT, Ok(Type::Bool)) => Ok(Type::Bool),
                (_, Err(e)) => Err(e),
                _ => Err(()), 
            },
            Expression::Factor(factor) => factor.try_into(),
            Expression::NotImplemented => Err(()),
        }
    }
}

fn try_parse_binop(op: BinOp, t: Type) {

}
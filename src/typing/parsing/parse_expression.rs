use colored::{Color, Colorize};

use crate::{ast::nodes::{BinOp, Expression, UnOp}, common::diagnostic::{Diagnostic, DiagnosticGravity}, typing::{Type, Typeable, TypingContext}};

impl Typeable for Expression {
    fn parse_type(&self, context: &mut TypingContext) -> Result<Type, ()> {
        match self {
            Expression::BINOP(e1, bin_op, e2) => try_parse_binop(self, e1.as_ref(), *bin_op, e2.as_ref(), context),
            Expression::UNOP(un_op, expr) => match (un_op, expr.as_ref().parse_type(context)) {
                (UnOp::NEG, Ok(Type::Int)) => Ok(Type::Int),
                (UnOp::NOT, Ok(Type::Bool)) => Ok(Type::Bool),
                _ => Err(()), 
            },
            Expression::Factor(factor) => factor.parse_type(context),
            Expression::NotImplemented => Err(()), // todo!()
        }
    }
}

fn try_parse_binop(root: &Expression, e1: &Expression, op: BinOp, e2: &Expression, context: &mut TypingContext) -> Result<Type, ()> {

    let t1_parsing = e1.parse_type(context);
    let t2_parsing = e2.parse_type(context);

    let (t1, t2) = match (t1_parsing, t2_parsing) {
        (Ok(t1), Ok(t2)) => (t1, t2),
        _ => return Err(()),
    };

    match op {
        BinOp::EQ
        | BinOp::NEQ => {
            if t1 != t2 && !(t1 == Type::Int && t2 == Type::Bool) 
                && !(t1 == Type::Bool && t2 == Type::Int) {
                match (&t1, &t2) {
                    (Type::Bool, Type::Int)
                    | (Type::Int, Type::Bool) => (),
                    _ => context.warnings.push(Diagnostic::dubious_comparison(root, &t1, &t2, false)),
                }    
            }
            return Ok(Type::Bool);
        }
        BinOp::AND
        | BinOp::OR => Ok(Type::Bool),
        BinOp::LESS
        | BinOp::LESSEQ
        | BinOp::GREATER
        | BinOp::GREATEREQ => {
            if t1 != t2 {
                context.errors.push(
                    Diagnostic::from_localizable_ref(
                        root, 
                        DiagnosticGravity::Error,
                        String::from("TypeError"),
                        format!(
                            "Cannot compare type {} with type {}",
                            format!("{}", t1).color(Color::Yellow),
                            format!("{}", t2).color(Color::Yellow)
                        )
                    )
                );
                return Err(());
            } else {
                return Ok(Type::Bool);
            }
        },
        BinOp::MULT
        | BinOp::DIV
        | BinOp::MOD
        | BinOp::SUB => {
            if t1 != Type::Int || t2 != Type::Int {
                context.errors.push(
                    Diagnostic::from_localizable_ref(
                        root, 
                        DiagnosticGravity::Error,
                        String::from("TypeError"),
                        format!(
                            "Incompatible types {} and {} for operand {}",
                            format!("{}", t1).color(Color::Yellow),
                            format!("{}", t2).color(Color::Yellow),
                            format!("{}", op).color(Color::Magenta)
                        )
                    )
                );
                return Err(());
            } else {
                return Ok(Type::Int);
            }
        }
        BinOp::ADD if t1 != t2 => {
            context.errors.push(
                Diagnostic::from_localizable_ref(
                    root, 
                    DiagnosticGravity::Error,
                    String::from("TypeError"),
                    format!(
                        "Incompatible types {} and {} for operand {}",
                        format!("{}", t1).color(Color::Yellow),
                        format!("{}", t2).color(Color::Yellow),
                        "+".color(Color::Magenta)
                    )
                )
            );
            return Err(());
        },
        BinOp::ADD => Ok(t1),
        BinOp::ACCESS => {
            if t2 == Type::Int {
                Ok(Type::Any)
            } else {
                context.errors.push(
                    Diagnostic::from_localizable_ref(
                        root, 
                        DiagnosticGravity::Error,
                        String::from("TypeError"),
                        format!(
                            "Invalid type {} for list index, must be an {}",
                            format!("{}", t2).color(Color::BrightRed),
                            "Int".color(Color::Yellow),
                        )
                    )
                );
                Err(())
            }
        },
    }
}

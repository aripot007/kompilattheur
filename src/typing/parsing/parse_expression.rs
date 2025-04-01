use colored::{Color, Colorize};

use crate::{ast::nodes::{BinOp, Expression, ExpressionKind, UnOp}, common::{diagnostic::{Diagnostic, DiagnosticGravity}, localizable::{localization_info, LocalizationInfo, Localizable}}, typing::{Type, Typeable, TypingContext}};

impl Typeable for Expression {
    fn parse_type(&mut self, context: &mut TypingContext) -> Result<Type, ()> {

        let localization = localization_info!(self);

        match &mut self.kind {
            ExpressionKind::BINOP(ref mut e1, bin_op, ref mut e2) => {
                match try_parse_binop(localization, e1.as_mut(), *bin_op, e2.as_mut(), context) {
                    Ok(t) => {
                        self.set_type(t.clone());
                        return Ok(t);
                    },
                    Err(_) => Err(()),
                }
            },
            ExpressionKind::UNOP(un_op, ref mut expr) => match (un_op, expr.as_mut().parse_type(context)) {
                (UnOp::NEG, Ok(Type::Int)) => Ok(Type::Int),
                (UnOp::NOT, Ok(Type::Bool)) => Ok(Type::Bool),
                _ => Err(()),
            },
            ExpressionKind::Factor(factor) => factor.parse_type(context),
            ExpressionKind::NotImplemented => Err(()), // todo!()
        }
    }
    
    fn is_typed(&self) -> bool {
        self.expr_type.is_some()
    }
    
    fn get_type(&self) -> &Type {
        self.expr_type.as_ref().unwrap()
    }
    
    fn get_type_opt(&self) -> Option<&Type> {
        match &self.expr_type {
            Some(t) => Some(t),
            None => None,
        }
    }
    
    fn set_type(&mut self, t: Type) {
        self.expr_type = Some(t);
    }
}

fn try_parse_binop(root_localization: LocalizationInfo, e1: &mut Expression, op: BinOp, e2: &mut Expression, context: &mut TypingContext) -> Result<Type, ()> {

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
                    _ => context.warnings.push(Diagnostic::dubious_comparison(&root_localization, &t1, &t2, false)),
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
                    Diagnostic::from_localizable(
                        root_localization, 
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
                    Diagnostic::from_localizable(
                        root_localization, 
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
                Diagnostic::from_localizable(
                    root_localization, 
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
                    Diagnostic::from_localizable(
                        root_localization, 
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

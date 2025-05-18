use colored::{Color, Colorize};

use crate::{
    ast::nodes::{BinOp, Expression, ExpressionKind, UnOp},
    common::{
        diagnostic::{Diagnostic, DiagnosticGravity},
        localizable::{localization_info, Localizable, LocalizationInfo},
    },
    typing::{Type, Typeable, TypingContext},
};

impl Typeable for Expression {
    fn parse_type(&mut self, context: &mut TypingContext) -> Result<Type, ()> {
        let localization = localization_info!(self);

        let res = match &mut self.kind {
            ExpressionKind::BINOP(ref mut e1, bin_op, ref mut e2) => {
                try_parse_binop(localization, e1.as_mut(), *bin_op, e2.as_mut(), context)
            }
            ExpressionKind::UNOP(un_op, ref mut expr) => {
                match (&un_op, expr.as_mut().parse_type(context)) {
                    // TODO: Adapt to weak types
                    (UnOp::NEG, Ok(Type::Int)) => Ok(Type::Int),
                    (UnOp::NOT, Ok(Type::Bool)) => Ok(Type::Bool),
                    (UnOp::NEG, Ok(Type::Weak(weak))) => {
                        let possible = weak.get_possible();
                        match weak.restrict(&[Type::Int]) {
                            Ok(t) => Ok(t),
                            Err(_) => {
                                context.errors.push(Diagnostic::invalid_unop_weak_type(
                                    expr.as_ref(),
                                    UnOp::NEG,
                                    &possible,
                                ));
                                Err(())
                            }
                        }
                    }
                    _ => {
                        context.errors.push(Diagnostic::invalid_unop_type(
                            &localization,
                            *un_op,
                            &expr.get_type(),
                        ));
                        Err(())
                    }
                }
            }
            ExpressionKind::Factor(ref mut factor) => factor.parse_type(context),
            ExpressionKind::NotImplemented => {
                context
                    .errors
                    .push(Diagnostic::not_implemented(&localization));
                Err(())
            }
        };

        if let Ok(t) = &res {
            self.set_type(t.clone());
        }
        return res;
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

fn try_parse_binop(
    root_localization: LocalizationInfo,
    e1: &mut Expression,
    op: BinOp,
    e2: &mut Expression,
    context: &mut TypingContext,
) -> Result<Type, ()> {
    let t1_parsing = e1.parse_type(context);
    let t2_parsing = e2.parse_type(context);

    let (t1, t2) = match (t1_parsing, t2_parsing) {
        (Ok(t1), Ok(t2)) => (t1, t2),
        _ => return Err(()),
    };

    match op {
        BinOp::EQ | BinOp::NEQ => {
            if t1 != t2
                && !(t1 == Type::Int && t2 == Type::Bool)
                && !(t1 == Type::Bool && t2 == Type::Int)
            {
                match (&t1, &t2) {
                    (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => (),
                    _ => context.warnings.push(Diagnostic::dubious_comparison(
                        &root_localization,
                        &t1,
                        &t2,
                        false,
                    )),
                }
            }
            return Ok(Type::Bool);
        }
        BinOp::AND | BinOp::OR => Ok(Type::Bool),
        BinOp::LESS | BinOp::LESSEQ | BinOp::GREATER | BinOp::GREATEREQ => {
            try_parse_comparison(root_localization, e1, t1, op, e2, t2, context)
        }
        BinOp::MULT | BinOp::DIV | BinOp::MOD | BinOp::SUB => {
            match (&t1, &t2) {
                (Type::Weak(w1), Type::Weak(w2)) => {
                    w1.intersection(&w2);
                    match w1.restrict(&[Type::Int]) {
                        Ok(_) => return Ok(Type::Int),
                        Err(_) => (),
                    }
                }
                (t, Type::Weak(weak)) | (Type::Weak(weak), t) if t.is_compatible(Type::Int) => {
                    match weak.restrict(&[Type::Int]) {
                        Ok(_) => return Ok(Type::Int),
                        Err(_) => (),
                    }
                }
                (t1, t2) if t1.is_compatible(Type::Int) && t2.is_compatible(Type::Int) => {
                    return Ok(Type::Int)
                }
                _ => (),
            }

            context.errors.push(Diagnostic::from_localizable(
                root_localization,
                DiagnosticGravity::Error,
                String::from("TypeError"),
                format!(
                    "Incompatible types {} and {} for operand {}",
                    format!("{}", t1).color(Color::Yellow),
                    format!("{}", t2).color(Color::Yellow),
                    format!("{}", op).color(Color::Magenta)
                ),
            ));
            return Err(());
        }
        BinOp::ADD if t1 != t2 => {
            context.errors.push(Diagnostic::from_localizable(
                root_localization,
                DiagnosticGravity::Error,
                String::from("TypeError"),
                format!(
                    "Incompatible types {} and {} for operand {}",
                    format!("{}", t1).color(Color::Yellow),
                    format!("{}", t2).color(Color::Yellow),
                    "+".color(Color::Magenta)
                ),
            ));
            return Err(());
        }
        BinOp::ADD => Ok(t1),
        BinOp::ACCESS => {
            if t2 == Type::Int {
                Ok(Type::Any)
            } else {
                context.errors.push(Diagnostic::from_localizable(
                    root_localization,
                    DiagnosticGravity::Error,
                    String::from("TypeError"),
                    format!(
                        "Invalid type {} for list index, must be an {}",
                        format!("{}", t2).color(Color::BrightRed),
                        "Int".color(Color::Yellow),
                    ),
                ));
                Err(())
            }
        }
    }
}

/// Parse comparison operators <, <=, > and >=
fn try_parse_comparison(
    root_localization: LocalizationInfo,
    e1: &Expression,
    e1_type: Type,
    op: BinOp,
    e2: &Expression,
    e2_type: Type,
    context: &mut TypingContext,
) -> Result<Type, ()> {
    if e1_type != e2_type {
        context.errors.push(Diagnostic::from_localizable(
            root_localization,
            DiagnosticGravity::Error,
            String::from("TypeError"),
            format!(
                "Cannot compare type {} with type {}",
                format!("{}", e1_type).color(Color::Yellow),
                format!("{}", e2_type).color(Color::Yellow)
            ),
        ));
        return Err(());
    } else {
        return Ok(Type::Bool);
    }
}

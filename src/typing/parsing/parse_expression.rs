use std::collections::HashSet;

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
        BinOp::AND | BinOp::OR => Ok(Type::Any), // AND and OR operators accept any type, for now we return Any to avoid complicated type parsing
        BinOp::LESS | BinOp::LESSEQ | BinOp::GREATER | BinOp::GREATEREQ => {
            try_parse_comparison(root_localization, t1, op, t2, context)
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
        BinOp::ADD => return try_parse_add(root_localization, t1, t2, context),
        BinOp::ACCESS => {
            // TODO(Aristide)
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

/// Parse add operation
fn try_parse_add(
    root_localization: LocalizationInfo,
    t1: Type,
    t2: Type,
    context: &mut TypingContext,
) -> Result<Type, ()> {
    if !t1.is_compatible(t2.clone()) {
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

    match (t1, t2) {
        (Type::Weak(w1), Type::Weak(w2)) => {
            w1.intersection(&w2);
            return Ok(Type::Weak(w1));
        }
        (Type::Any, t) | (t, Type::Any) => return Ok(t),
        (Type::Weak(weak), t) | (t, Type::Weak(weak)) => {
            weak.restrict(&[t.clone()])
                .expect("Weak restriction should not fail if its compatible");
            return Ok(t);
        }
        (t1, t2) if t1 == t2 => return Ok(t1),
        _ => unreachable!(),
    }
}

/// Parse comparison operators <, <=, > and >=
fn try_parse_comparison(
    root_localization: LocalizationInfo,
    t1: Type,
    op: BinOp,
    t2: Type,
    context: &mut TypingContext,
) -> Result<Type, ()> {
    match (&t1, &t2) {
        (Type::Any, _) | (_, Type::Any) => return Ok(Type::Bool),
        (Type::Weak(w1), Type::Weak(w2)) => {
            let id_1 = w1.get_id();
            let id_2 = w2.get_id();

            // Used for debug
            let types_1: Vec<Type> = w1.get_possible();
            let types_2: Vec<Type> = w2.get_possible();

            let mut possible_1: HashSet<Type> = HashSet::from_iter(types_1.clone());
            let mut possible_2: HashSet<Type> = HashSet::from_iter(types_2.clone());

            // Remove None types as they are not allowed in comparisons
            possible_1.remove(&Type::None);
            possible_2.remove(&Type::None);

            let arithmetic_types: HashSet<Type> = HashSet::from_iter([Type::Bool, Type::Int]);

            // If both intersections are not empty, we can do arithmetic comparison between the 2 weaks
            if arithmetic_types.intersection(&possible_1).count() > 0
                && arithmetic_types.intersection(&possible_2).count() > 0
            {
                // We restrict both types without making them equal, and without deleting the arithmetic types
                let new_possible_1: Vec<Type> = possible_1
                    .intersection(&possible_2.union(&arithmetic_types).cloned().collect())
                    .cloned()
                    .collect();
                let ok = w1.restrict(&new_possible_1).is_ok();

                let new_possible_2: Vec<Type> = possible_2
                    .intersection(&possible_1.union(&arithmetic_types).cloned().collect())
                    .cloned()
                    .collect();

                let ok = w2.restrict(&new_possible_2).is_ok() || ok;

                if ok {
                    return Ok(Type::Bool);
                }

                context.errors.push(Diagnostic::invalid_binop_weak_type(
                    &root_localization,
                    op,
                    id_1,
                    &types_1,
                    id_2,
                    &types_2,
                ));
                return Err(());
            } else {
                // Else, we can only compare types that are the same, but we need to remove the None type
                w1.intersection(w2);
                let _ = w1.remove(Type::None); // Ignore the result as we check it after
            }

            if w1.get_possible().len() != 0 {
                return Ok(Type::Bool);
            } else {
                context.errors.push(Diagnostic::invalid_binop_weak_type(
                    &root_localization,
                    op,
                    id_1,
                    &types_1,
                    id_2,
                    &types_2,
                ));
                return Err(());
            }
        }
        (Type::String, t) | (t, Type::String) => {
            if t.is_compatible(Type::String) {
                if let Type::Weak(weak) = t {
                    weak.restrict(&[Type::String])
                        .expect("Restriction should not fail since weak is compatible");
                }
                return Ok(Type::Bool);
            }
        }
        (Type::List, t) | (t, Type::List) => {
            if t.is_compatible(Type::List) {
                if let Type::Weak(weak) = t {
                    weak.restrict(&[Type::List])
                        .expect("Restriction should not fail since weak is compatible");
                }
                return Ok(Type::Bool);
            }
        }
        (Type::Int, t) | (Type::Bool, t) | (t, Type::Int) | (t, Type::Bool) => {
            if t.is_compatible(Type::Int) || t.is_compatible(Type::Bool) {
                if let Type::Weak(weak) = t {
                    weak.restrict(&[Type::Int, Type::Bool])
                        .expect("Restriction should not fail since weak is compatible");
                }
            }
        }
        _ => (),
    }
    context.errors.push(Diagnostic::from_localizable(
        root_localization,
        DiagnosticGravity::Error,
        String::from("TypeError"),
        format!(
            "Cannot compare type {} with type {}",
            format!("{}", t1).color(Color::Yellow),
            format!("{}", t2).color(Color::Yellow)
        ),
    ));
    return Err(());
}

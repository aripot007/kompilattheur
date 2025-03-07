use colored::{Color, Colorize};

use crate::{ast::nodes::{Ast, BinOp, Expression, Factor, UnOp}, common::{diagnostic::{Diagnostic, DiagnosticGravity}, localizable::Localizable}};

use super::types::Type;

type TypingErrors = Vec<Diagnostic>;

macro_rules! err_single_diag {
    ($diag: expr) => {
        Err(vec![$diag])
    };
}

impl TryFrom<Ast> for Type {
    type Error = TypingErrors;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok((t, _)) => Ok(t),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<Ast> for (Type, Vec<Diagnostic>) {

    type Error = TypingErrors;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        return match value {
            Ast::Expression(expression) => todo!(),
            Ast::Conditional(conditional) => todo!(),
            Ast::Factor(factor) => factor.try_into(),
            Ast::Param(param) => todo!(),
            Ast::Statement(statement) => todo!(),
            Ast::Assign(_)
            | Ast::For(_) => Ok((Type::None, Vec::new())),
            Ast::Def(def) => todo!(),
            Ast::Defs(_)
            | Ast::Block(_)
            | Ast::Root(_) => panic!("Cannot infer type of Defs, Block or Root node")
        };
    }
}

impl TryFrom<Factor> for Type {
    type Error = TypingErrors;

    fn try_from(value: Factor) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok((t, _)) => Ok(t),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<Factor> for (Type, Vec<Diagnostic>) {
    type Error = TypingErrors;

    fn try_from(value: Factor) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&Factor> for Type {
    type Error = TypingErrors;

    fn try_from(value: &Factor) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok((t, _)) => Ok(t),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<&Factor> for (Type, Vec<Diagnostic>) {
    type Error = TypingErrors;

    fn try_from(factor: &Factor) -> Result<Self, Self::Error> {
        return match factor {
            Factor::Integer(_) => Ok((Type::Int, Vec::new())),
            Factor::String(_) => Ok((Type::String, Vec::new())),
            Factor::True(_)
            | Factor::False(_) => Ok((Type::Bool, Vec::new())),
            Factor::None(_) => Ok((Type::None, Vec::new())),
            Factor::Identifier(_) => todo!(), // Get or add to tds
            Factor::List(_) => Ok((Type::List, Vec::new())),
            Factor::Expr(expr) => expr.as_ref().try_into(),
            Factor::Call { identifier, args } => todo!(),// Get or add to tds
        };
    }
}

impl TryFrom<&Expression> for Type {
    type Error = TypingErrors;

    fn try_from(value: &Expression) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok((t, _)) => Ok(t),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<&Expression> for (Type, Vec<Diagnostic>) {
    type Error = TypingErrors;

    fn try_from(value: &Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::BINOP(e1, bin_op, e2) => try_parse_binop(value, e1.as_ref(), *bin_op, e2.as_ref()),
            Expression::UNOP(un_op, expr) => match (un_op, expr.as_ref().try_into()) {
                (UnOp::NEG, Ok(Type::Int)) => Ok((Type::Int, Vec::new())),
                (UnOp::NOT, Ok(Type::Bool)) => Ok((Type::Bool, Vec::new())),
                (_, Err(e)) => Err(e),
                _ => Err(Vec::new()), 
            },
            Expression::Factor(factor) => factor.try_into(),
            Expression::NotImplemented => Err(Vec::new()),
        }
    }
}

fn try_parse_binop(root: &Expression, e1: &Expression, op: BinOp, e2: &Expression) -> Result<(Type, Vec<Diagnostic>), TypingErrors> {
    
    let mut errors: Vec<Diagnostic> = Vec::new();

    let t1_parsing = match e1.try_into() {
        Ok(t) => Some(t),
        Err(e) => {
            let mut e = e;
            errors.append(&mut e);
            None
        } 
    };

    let t2_parsing = match e2.try_into() {
        Ok(t) => Some(t),
        Err(e) => {
            let mut e = e;
            errors.append(&mut e);
            None
        } 
    };

    let ((t1, w1), (t2, w2)) = match (t1_parsing, t2_parsing) {
        (Some(t1), Some(t2)) => (t1, t2),
        _ => return Err(errors)
    };

    let mut warnings = w1;
    let mut w2 = w2;

    warnings.append(&mut w2);

    match op {
        BinOp::EQ
        | BinOp::NEQ => {
            if t1 != t2 && !(t1 == Type::Int && t2 == Type::Bool) 
                && !(t1 == Type::Bool && t2 == Type::Int) {
                match (&t1, &t2) {
                    (Type::Bool, Type::Int)
                    | (Type::Int, Type::Bool) => (),
                    _ => warnings.push(Diagnostic::dubious_comparison(root, &t1, &t2, false)),
                }    
            }
            return Ok((Type::Bool, warnings));
        }
        BinOp::AND
        | BinOp::OR => Ok((Type::Bool, warnings)),
        BinOp::LESS
        | BinOp::LESSEQ
        | BinOp::GREATER
        | BinOp::GREATEREQ => {
            if t1 != t2 {
                return err_single_diag!(
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
                )
            } else {
                return Ok((Type::Bool, warnings));
            }
        },
        BinOp::MULT
        | BinOp::DIV
        | BinOp::MOD
        | BinOp::SUB => {
            if t1 != Type::Int || t2 != Type::Int {
                return err_single_diag!(
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
                )
            } else {
                return Ok((Type::Int, warnings));
            }
        }
        BinOp::ADD if t1 != t2 => err_single_diag!(
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
        ),
        BinOp::ADD => Ok((t1, warnings)),
        BinOp::ACCESS => {
            if t2 == Type::Int {
                Ok((Type::Any, warnings))
            } else {
                err_single_diag!(
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
                )
            }
        },
    }
}
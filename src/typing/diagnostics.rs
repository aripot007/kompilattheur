use colored::{Color, Colorize};

use super::Type;
/// Defines all diagnostics used in the typing system
use crate::{
    ast::nodes::UnOp,
    common::{
        diagnostic::{Diagnostic, DiagnosticGravity},
        localizable::Localizable,
    },
};

impl Diagnostic {
    /// Dubious comparison between elements of different types that always evaluate to the same value
    pub(super) fn dubious_comparison<T: Localizable>(
        root: &T,
        t1: &Type,
        t2: &Type,
        value: bool,
    ) -> Self {
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Warning,
            String::from("DubiousComparison"),
            format!(
                "Comparison of types {} and {} always return {}",
                format!("{}", t1).color(Color::Yellow),
                format!("{}", t2).color(Color::Yellow),
                value
            ),
        )
    }

    /// Incompatible type for an expression
    pub(super) fn incompatible_type<T: Localizable>(
        root: &T,
        t1: &Type,
        expected: &[Type],
    ) -> Self {
        let expected_string = expected
            .iter()
            .map(Type::to_string)
            .collect::<Vec<String>>()
            .join(", ");

        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("TypeError"),
            format!(
                "Expected type {}, but got {}",
                format!("{}", t1).color(Color::Red),
                expected_string.color(Color::Yellow)
            ),
        )
    }

    /// Invalid type for unary operation
    pub(super) fn invalid_unop_type<T: Localizable>(
        root: &T,
        operator: UnOp,
        expression_type: &Type,
    ) -> Self {
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("TypeError"),
            format!(
                "Invalid type {} for unary operator {}",
                format!("{}", expression_type).color(Color::Red),
                operator.to_string().color(Color::Magenta)
            ),
        )
    }

    /// Invalid type for unary operation
    pub(super) fn invalid_unop_weak_type<T: Localizable>(
        root: &T,
        operator: UnOp,
        weak_types: &[Type],
    ) -> Self {
        let strs: Vec<String> = weak_types.iter().map(|t| t.to_string()).collect();
        let types = format!("weak({})", strs.join(", "));
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("TypeError"),
            format!(
                "Invalid type {} for unary operator {}",
                format!("{}", types).color(Color::Red),
                operator.to_string().color(Color::Magenta)
            ),
        )
    }

    /// Not implemented expression
    pub(super) fn not_implemented<T: Localizable>(root: &T) -> Self {
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("NotImplemented"),
            format!("Expression not implemented"),
        )
    }

    /// Return statement outside of a function
    pub(super) fn return_outside_function<T: Localizable>(root: &T) -> Self {
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("ReturnError"),
            format!("Return statement found outside of a function"),
        )
    }

    /// Unknown symbol
    pub(super) fn unknown_symbol<T: Localizable>(root: &T, name: &String) -> Self {
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("UnknownSymbol"),
            format!("Unknown symbol {}", format!("{}", name).color(Color::Red),),
        )
    }
}

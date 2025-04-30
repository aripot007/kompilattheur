use colored::{Color, Colorize};

use super::Type;
/// Defines all diagnostics used in the typing system
use crate::common::{
    diagnostic::{Diagnostic, DiagnosticGravity},
    localizable::Localizable,
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

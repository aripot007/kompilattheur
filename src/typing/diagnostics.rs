use colored::{Color, Colorize};

/// Defines all diagnostics used in the typing system

use crate::{ast::nodes::Expression, common::{diagnostic::{Diagnostic, DiagnosticGravity}, localizable::Localizable}};
use super::Type;


impl Diagnostic {

    /// Dubious comparison between elements of different types that always evaluate to the same value
    pub(super) fn dubious_comparison(root: &Expression, t1: &Type, t2: &Type, value: bool) -> Self {
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

    /// Unknown symbol
    pub(super) fn unknown_symbol<T: Localizable>(root: &T, name: &String) -> Self {
        Diagnostic::from_localizable_ref(
            root,
            DiagnosticGravity::Error,
            String::from("UnknownSymbol"),
            format!(
                "Unknown symbol {}",
                format!("{}", name).color(Color::Red),
            ),
        )
    }
}

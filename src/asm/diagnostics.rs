use colored::{Color, Colorize};

use crate::{ast::nodes::{Assign, AstNode}, common::diagnostic::{Diagnostic, DiagnosticGravity}};

impl Diagnostic {
    
    /// Unimplemented LLVM for an ast node
    pub(super) fn unimplemented_llvm<A: AstNode>(ast_node: &A) -> Self {
        Diagnostic::from_localizable_ref(
            ast_node, 
            DiagnosticGravity::Error, 
            String::from("LLVMNotImplemented"),
            format!(
                "LLVM Generation for ast node {} is not yet implemented",
                ast_node.get_string_repr().color(Color::Red)
            )
        )
    }

    /// Destination expression does not resolve to a valid memory address
    pub(super) fn invalid_destination_expr<A: AstNode>(expr: &A) -> Self {
        Diagnostic::from_localizable_ref(
            expr, 
            DiagnosticGravity::Error, 
            String::from("InvalidDestination"),
            format!(
                "The expression '{}' does not compute to a valid storage location",
                expr.get_string_repr().color(Color::Red)
            )
        )
    }

    /// Expression result from an assign not stored
    pub(super) fn discarded_assign_result(assign: &Assign) -> Self {
        Diagnostic::from_localizable_ref(
            assign, 
            DiagnosticGravity::Warning, 
            String::from("DiscardedValue"),
            format!(
                "The result of this assignation is discarded, as '{}' is not a destination",
                assign.destination.get_string_repr().color(Color::Yellow)
            )
        )
    }
}
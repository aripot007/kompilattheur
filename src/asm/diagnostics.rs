use colored::{Color, Colorize};

use crate::{ast::nodes::AstNode, common::diagnostic::{Diagnostic, DiagnosticGravity}};

impl Diagnostic {
    
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
}
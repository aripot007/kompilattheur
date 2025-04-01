use inkwell::values::StructValue;

use crate::{asm::{codegen::CodeGen, llvm::LLVMCodegenError, llvm::print::*, InternalFuctions}, ast::nodes::{Block, Expression}, common::diagnostic::{Diagnostic, DiagnosticGravity}, typing::{Type, Typeable}};
use super::llvm_compute_expr;

pub fn llvm_from_block<'ctx>(block: &Block, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    let mut error = false;

    for stmt in &block.statements {

        match stmt {
            crate::ast::nodes::Statement::Print(expr) => llvm_from_print(expr, cg)?,
            crate::ast::nodes::Statement::Return(_)
            | crate::ast::nodes::Statement::For(_)
            | crate::ast::nodes::Statement::Conditional(_)
            | crate::ast::nodes::Statement::Assign(_)
            | crate::ast::nodes::Statement::Expr(_)
            | crate::ast::nodes::Statement::NotImplemented => {
                cg.errors.push(Diagnostic::unimplemented_llvm(stmt));
                error = true;
            }
        }

    }

    if error {
        return Err(());
    }
    
    return Ok(());
}


fn llvm_from_print<'ctx>(expr: &Expression, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    let expr_value: StructValue<'ctx> = llvm_compute_expr(expr, cg)?;

    match expr.get_type() {
        Type::None => print_none_value(&expr_value, cg),
        Type::Bool => print_bool_value(&expr_value, cg),
        Type::Int => print_int_value(&expr_value, cg),
        Type::String => print_string_value(&expr_value, cg),
        _ => {
            cg.errors.push(Diagnostic::from_localizable_ref(
                expr,
                DiagnosticGravity::Error,
                String::from("UnimplementedLLVM"),
                String::from("print string pls")
            ));
            return Err(());
        }
    }

    return Ok(());
}

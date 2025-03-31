use inkwell::values::StructValue;

use crate::{asm::{codegen::CodeGen, InternalFuctions}, ast::nodes::{Block, Expression}, common::diagnostic::{Diagnostic, DiagnosticGravity}};
use super::{llvm_compute_expr, LLVMCodegenError};

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

    // tkt ça arrive soon
    if /* expr.type != Type::String */ false {
        cg.errors.push(Diagnostic::from_localizable_ref(
            expr,
            DiagnosticGravity::Error,
            String::from("UnimplementedLLVM"),
            String::from("print string pls")
        ));
        return Err(());
    }

    // TODO : Compute and store value of expression in register
    // For now its a string

    let expr_value: StructValue<'ctx> = llvm_compute_expr(expr, cg)?;

    // TODO : C'est des string tkt (et onfera une fonction print plus tard)
    let puts = cg.module.get_function(InternalFuctions::Puts.into()).unwrap();

    cg.builder
        .build_call(puts, &[expr_value.get_field_at_index(1).unwrap().into()], "printf_call")
        .unwrap();

    return Ok(());
}

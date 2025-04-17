use crate::{asm::{codegen::CodeGen, llvm::{print::*, smolvar::SmolVar, LLVMCodegenError}}, ast::nodes::{AstNode, Block, Expression, Statement}, common::diagnostic::{Diagnostic, DiagnosticGravity}, typing::{Type, Typeable}};
use super::llvm_compute_expr;

pub fn llvm_from_block<'ctx>(block: &Block, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    let mut error = false;

    for stmt in &block.statements {

        match stmt {
            Statement::Print(expr) => llvm_from_print(expr, cg)?,
            Statement::Return(_)
            | Statement::For(_)
            | Statement::Conditional(_)
            | Statement::Assign(_)
            | Statement::Expr(_)
            | Statement::NotImplemented => {
                cg.errors.push(Diagnostic::unimplemented_llvm(stmt));
                error = true;
            }
        }

    }

    if error {
        return Err(LLVMCodegenError::Unimplemented(String::from("Block")));
    }
    
    return Ok(());
}


fn llvm_from_print<'ctx>(expr: &Expression, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    let expr_value: SmolVar<'ctx> = llvm_compute_expr(expr, cg)?;

    match expr.get_type() {
        Type::None => print_none_value(&expr_value, cg)?,
        Type::Bool => print_bool_value(&expr_value, cg)?,
        Type::Int => print_int_value(&expr_value, cg)?,
        Type::String => print_string_value(&expr_value, cg)?,
        Type::List => print_list_value(&expr_value, cg)?,
        Type::Weak(_)
        | Type::Any => print_any_value(&expr_value, cg)?,
        _ => {
                cg.errors.push(Diagnostic::from_localizable_ref(
                    expr,
                    DiagnosticGravity::Error,
                    String::from("UnimplementedLLVM"),
                    format!("Unimplemented llvm for expression {}", expr.get_string_repr())
                ));
                return Err(LLVMCodegenError::Unimplemented(expr.get_string_repr()));
            }
    };

    return Ok(());
}

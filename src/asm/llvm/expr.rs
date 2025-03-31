use inkwell::values::StructValue;

use crate::{asm::codegen::CodeGen, ast::nodes::Expression, common::diagnostic::Diagnostic};
use super::{llvm_compute_factor, LLVMCodegenError};

pub fn llvm_compute_expr<'ctx>(expr: &Expression, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    match expr {
        Expression::BINOP(_e1, _bin_op, _e2) => (),
        Expression::UNOP(_un_op, _e1) => (),
        Expression::Factor(factor) => return llvm_compute_factor(factor, cg),
        Expression::NotImplemented => (),
    }

    cg.errors.push(Diagnostic::unimplemented_llvm(expr));

    return Err(());
}

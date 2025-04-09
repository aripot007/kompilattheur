use inkwell::values::StructValue;

use crate::{asm::{codegen::CodeGen, llvm::LLVMCodegenError}, ast::nodes::{Expression, ExpressionKind}, common::diagnostic::Diagnostic};
use super::super::llvm_compute_factor;

pub fn llvm_compute_expr<'ctx>(expr: &Expression, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    match &expr.kind {
        ExpressionKind::BINOP(_e1, _bin_op, _e2) => (),
        ExpressionKind::UNOP(_un_op, _e1) => (),
        ExpressionKind::Factor(factor) => return llvm_compute_factor(factor, cg),
        ExpressionKind::NotImplemented => (),
    }

    cg.errors.push(Diagnostic::unimplemented_llvm(expr));

    return Err(());
}

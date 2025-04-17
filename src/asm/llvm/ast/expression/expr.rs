use crate::{asm::{codegen::CodeGen, llvm::{smolvar::SmolVar, LLVMCodegenError}}, ast::nodes::{AstNode, Expression, ExpressionKind}, common::diagnostic::Diagnostic};
use super::{super::llvm_compute_factor, llvm_compute_binop, llvm_compute_unop};

pub fn llvm_compute_expr<'ctx>(expr: &Expression, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {

    match &expr.kind {
        ExpressionKind::BINOP(e1, op, e2) => return llvm_compute_binop(e1, op, e2, cg),
        ExpressionKind::UNOP(op, e1) => return llvm_compute_unop(op, e1, cg),
        ExpressionKind::Factor(factor) => return llvm_compute_factor(factor, cg),
        ExpressionKind::NotImplemented => (),
    }

    cg.errors.push(Diagnostic::unimplemented_llvm(expr));

    return Err(LLVMCodegenError::Unimplemented(expr.get_string_repr()));
}

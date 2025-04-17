use crate::{asm::{codegen::CodeGen, llvm::{smolvar::SmolVar, LLVMCodegenError}}, ast::nodes::{BinOp, Expression}};


pub fn llvm_compute_binop<'ctx>(e1: &Expression, op: &BinOp, e2: &Expression, cg: &CodeGen<'ctx>) ->  Result<SmolVar<'ctx>, LLVMCodegenError> {
    todo!()
}
use inkwell::values::StructValue;

use crate::{asm::{codegen::CodeGen, llvm::LLVMCodegenError}, ast::nodes::{BinOp, Expression, UnOp}};


pub fn llvm_compute_binop<'ctx>(e1: &Expression, op: &BinOp, e2: &Expression, cg: &CodeGen<'ctx>) ->  Result<StructValue<'ctx>, LLVMCodegenError> {
    todo!()
}
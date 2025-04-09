use inkwell::values::StructValue;

use crate::{asm::{codegen::CodeGen, llvm::LLVMCodegenError}, ast::nodes::{Expression, UnOp}};


pub fn llvm_compute_unop<'ctx>(op: &UnOp, expr: &Expression, cg: &CodeGen<'ctx>) ->  Result<StructValue<'ctx>, LLVMCodegenError> {
    todo!()
}
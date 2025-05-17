use super::llvm_compute_expr;
use crate::{
    asm::{codegen::CodeGen, llvm::LLVMCodegenError},
    ast::nodes::Expression,
};

pub fn llvm_from_return<'ctx>(
    expr: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let smol_var = llvm_compute_expr(expr, cg)?;

    cg.builder.build_return(Some(&smol_var))?;

    return Ok(());
}

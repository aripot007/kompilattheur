use crate::{asm::{codegen::CodeGen, llvm::LLVMCodegenError}, ast::nodes::Root};
use super::llvm_from_block;

pub fn llvm_from_root(root: &Root, cg: &mut CodeGen) -> Result<(), LLVMCodegenError> {

    // llvm from defs
    llvm_from_block(&root.block, cg)?;
    
    // Return 0
    let i32_type = cg.context.i32_type();
    let ret_val = i32_type.const_int(0, false);
    cg.builder.build_return(Some(&ret_val)).unwrap();

    return Ok(());
}

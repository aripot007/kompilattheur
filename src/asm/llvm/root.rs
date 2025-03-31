use crate::{asm::codegen::CodeGen, ast::nodes::Root};
use super::{llvm_from_block, LLVMCodegenError};

pub fn llvm_from_root(root: &Root, cg: &mut CodeGen) -> Result<(), LLVMCodegenError> {

    // Add main function entry point
    let i32_type = cg.context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = cg.module.add_function("main", fn_type, None);
    let basic_block = cg.context.append_basic_block(function, "entry");

    cg.builder.position_at_end(basic_block);

    // llvm from defs
    llvm_from_block(&root.block, cg)?;

    // Return 0
    let ret_val = i32_type.const_int(0, false);
    cg.builder.build_return(Some(&ret_val)).unwrap();

    return Ok(());
}

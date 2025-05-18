use crate::asm::codegen::CodeGen;

use super::LLVMCodegenError;

pub fn init_internal_range_function<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    // TODO: implement range()
    // range(n) -> [0, 1, ... n-1]

    Ok(())
}

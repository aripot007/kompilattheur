use crate::asm::codegen::CodeGen;

use super::LLVMCodegenError;

pub fn init_internal_list_function<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    // TODO: implement list()
    // assert type before exec
    // list(range(n)) -> [0, 1, ... n-1],
    // list([4, 5]) -> [4, 5],
    // list(non list) -> error

    Ok(())
}

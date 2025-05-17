use crate::asm::{codegen::CodeGen, llvm::LLVMCodegenError};
use crate::ast::nodes::Defs;

use super::llvm_from_block;

pub fn llvm_from_defs<'ctx>(defs: &Defs, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    for def in &defs.defs {
        let name = def.identifier.element.name.clone();
        let var_type = cg.smolpp_types.dynamic_type;

        // Register the function in the module
        let func_type = var_type.fn_type(&vec![var_type.into(); def.params.len()], false);
        let function = cg.module.add_function(
            format!("__smolpp_user_f_{}", name).as_str(),
            func_type,
            None,
        );

        // Build the function
        let entry = cg.context.append_basic_block(function, "function_entry");

        // Switch builder to the function block
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        llvm_from_block(&def.block, cg)?;

        // Return builder to main block
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
    }

    return Ok(());
}

use crate::{
    asm::{codegen::CodeGen, get_internal_func, llvm::LLVMCodegenError, InternalFuctions},
    ast::nodes::While,
};

use super::{llvm_compute_expr, llvm_from_block};

pub fn llvm_from_while_loop<'ctx>(
    while_loop: &While,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Get parent block
    let parent_block = cg.builder.get_insert_block().unwrap();

    let header_block = cg
        .context
        .insert_basic_block_after(parent_block, "while_header");
    let loop_block = cg
        .context
        .insert_basic_block_after(header_block, "while_block");
    let exit_block = cg
        .context
        .insert_basic_block_after(loop_block, "exit_block");

    cg.builder.build_unconditional_branch(header_block)?;

    cg.builder.position_at_end(header_block);

    let cdt_val = llvm_compute_expr(&while_loop.condition, cg)?;

    let ret_val = cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::BoolCast),
        &[cdt_val.into()],
        "cast_to_bool",
    )?;
    let cdt = ret_val.try_as_basic_value().unwrap_left().into_int_value();

    cg.builder
        .build_conditional_branch(cdt, loop_block, exit_block)?;

    cg.builder.position_at_end(loop_block);

    let branched = llvm_from_block(&while_loop.block, cg)?;

    if !branched {
        cg.builder.build_unconditional_branch(header_block)?;
    }

    cg.builder.position_at_end(exit_block);

    return Ok(());
}

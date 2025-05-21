use inkwell::{
    basic_block::BasicBlock,
    types::BasicType,
    values::{IntValue, PointerValue},
    IntPredicate,
};

use crate::asm::{codegen::CodeGen, LLVMCodegenError};

/// Build a memcpy subroutine
pub fn llvm_build_memcpy<'ctx, BT>(
    len: IntValue<'ctx>,
    src_array_ptr: PointerValue<'ctx>,
    dest_array_ptr: PointerValue<'ctx>,
    typ: BT,
    cg: &mut CodeGen<'ctx>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    BT: BasicType<'ctx> + Clone,
{
    let current_block = cg.builder.get_insert_block().unwrap();
    let for_loop_block = cg
        .context
        .insert_basic_block_after(current_block, "for_loop_block");
    let end_block = cg
        .context
        .insert_basic_block_after(current_block, "end_block");

    let i64_type = cg.context.i64_type();

    let index = i64_type.const_zero();
    let index_ptr = cg.builder.build_alloca(i64_type, "index_ptr")?;
    cg.builder.build_store(index_ptr, index)?;

    let empty_cdt =
        cg.builder
            .build_int_compare(IntPredicate::EQ, len, i64_type.const_zero(), "empty_cdt")?;

    cg.builder
        .build_conditional_branch(empty_cdt, end_block, for_loop_block)?;

    cg.builder.position_at_end(for_loop_block);

    let index_val = cg
        .builder
        .build_load(i64_type, index_ptr, "index")?
        .into_int_value();

    let src_elt_ptr = unsafe {
        cg.builder.build_gep(
            typ.clone(),
            src_array_ptr,
            &[index_val.into()],
            "src_elt_ptr",
        )
    }?;

    let dest_elt_ptr = unsafe {
        cg.builder.build_gep(
            typ.clone(),
            dest_array_ptr,
            &[index_val.into()],
            "dest_elt_ptr",
        )
    }?;

    let elt = cg.builder.build_load(typ.clone(), src_elt_ptr, "elt")?;
    cg.builder.build_store(dest_elt_ptr, elt)?;

    // Increment counter
    let index_val =
        cg.builder
            .build_int_add(index_val, i64_type.const_int(1, false), "incremented_index")?;
    cg.builder.build_store(index_ptr, index_val)?;

    let break_cdt = cg
        .builder
        .build_int_compare(IntPredicate::ULT, index_val, len, "break_cdt")?;

    cg.builder
        .build_conditional_branch(break_cdt, for_loop_block, end_block)?;

    cg.builder.position_at_end(end_block);

    return Ok(end_block);
}

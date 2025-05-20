use core::panic;

use crate::{
    asm::{
        codegen::CodeGen,
        get_internal_func,
        llvm::{smolvar::SmolVar, LLVMCodegenError},
        InternalFuctions,
    },
    ast::nodes::{BinOp, Expression},
};

use super::llvm_compute_expr;

pub fn llvm_compute_and_or<'ctx>(
    e1: &Expression,
    op: &BinOp,
    e2: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let value1 = llvm_compute_expr(e1, cg)?;

    // Cast e1_value to bool using the bool_cast_internal_function
    let call_boolean_llvm_value = cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::BoolCast),
        &[value1.into()],
        "and_or_bool_cast_call",
    )?;

    let boolean_llvm_value = call_boolean_llvm_value
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();
    // Branch if the types are equal
    let parent_block = cg.builder.get_insert_block().unwrap();
    let compute_right_block = cg
        .context
        .insert_basic_block_after(parent_block, "compute_right_block");
    let finish_block = cg
        .context
        .insert_basic_block_after(compute_right_block, "finish_block");

    match op {
        BinOp::AND => {
            cg.builder.build_conditional_branch(
                boolean_llvm_value,
                compute_right_block,
                finish_block,
            )?;
        }
        BinOp::OR => {
            cg.builder.build_conditional_branch(
                boolean_llvm_value,
                finish_block,
                compute_right_block,
            )?;
        }
        _ => panic!("Invalid operator for AND/OR computation"),
    }

    cg.builder.position_at_end(compute_right_block);

    let value2 = llvm_compute_expr(e2, cg)?;
    let block_right = cg.builder.get_insert_block().unwrap();

    cg.builder.build_unconditional_branch(finish_block)?;

    cg.builder.position_at_end(finish_block);

    let phi = cg
        .builder
        .build_phi(cg.smolpp_types.dynamic_type, "llvm_compute_and_or_phi")?;
    phi.add_incoming(&[(&value1, parent_block), (&value2, block_right)]);

    return Ok(phi.as_basic_value().into_struct_value());
}

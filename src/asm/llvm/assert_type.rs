use inkwell::{basic_block::BasicBlock, values::{IntValue, StructValue}, IntPredicate};

use crate::{asm::{codegen::CodeGen, internal_global_constants::RuntimeErrorMsg, llvm::panic::smolpp_panic}, typing::Type};

use super::{smolvar::SmolVar, LLVMCodegenError};


/// Generate LLVM to assert the type of a variable at runtime
/// The runtime type must be exactly `valtype`, weak types will not match
/// if they have other possible types.
pub fn assert_type<'ctx>(valtype: Type, value: &SmolVar<'ctx>, cg: &CodeGen<'ctx>, msg: Option<String>) -> Result<BasicBlock<'ctx>, LLVMCodegenError> {

    let msg = match msg {
        Some(s) => s,
        None => format!("Expected type {} ({}), got %s", valtype, valtype.get_bitmask()),
    };

    let type_field = cg.get_variable_type(*value)?;
    let expected_type = cg.context.i8_type().const_int(valtype.get_bitmask() as u64, false);

    let cdt = cg.builder.build_int_compare(IntPredicate::EQ, expected_type, type_field, format!("assert_type_{}", valtype).as_str())?;

    return create_assert_type_branch(cdt, cg, msg);
}

/// Generate LLVM to assert the type of a variable at runtime
/// The runtime type must be one of the types in `types`.
pub fn assert_type_oneof<'ctx>(types: &[Type], value: &SmolVar<'ctx>, cg: &CodeGen<'ctx>, msg: Option<String>) -> Result<BasicBlock<'ctx>, LLVMCodegenError> {

    let expected_bitmask: u8 = types.iter().map(Type::get_bitmask).reduce(|acc, bitmask| acc | bitmask).expect("Cannot assert empty type");

    let msg = match msg {
        Some(s) => s,
        None => {
            let types_str: String = types.iter().map(Type::to_string).collect::<Vec<_>>().join(", ");
            format!("Expected type {} ({:#b}), got %s", types_str, expected_bitmask)
        },
    };

    // _ separated list of accepted types
    let expected_types_str: String = types.iter().map(Type::to_string).collect::<Vec<_>>().join("_");

    let type_field = cg.get_variable_type(*value)?;
    let expected_type = cg.context.i8_type().const_int(expected_bitmask as u64, false);

    let cdt = cg.builder.build_and(expected_type, type_field, format!("assert_type_oneof_{}", expected_types_str).as_str())?;

    return create_assert_type_branch(cdt, cg, msg);
}

/// Create the conditional branch for type assertion.
/// If `cdt` is true, the type check is considered successful.
/// If its false, the programs print an error message and exits.
/// Returns the basic block after the branch
fn create_assert_type_branch<'ctx>(cdt: IntValue<'ctx>, cg: &CodeGen<'ctx>, msg: String) -> Result<BasicBlock<'ctx>, LLVMCodegenError> {

    let msg_str = cg.context.const_string(msg.as_bytes(), true);

    // Create panic block and continuation block
    let then_block = cg.context.append_basic_block(cg.current_function, "ok");
    let panic_block = cg.context.append_basic_block(cg.current_function, "panic");
    let merge_block = cg.context.append_basic_block(cg.current_function, "end");

    // Conditional branch
    cg.builder.build_conditional_branch(cdt, then_block, panic_block)?;

    // "Then" block : type is ok
    cg.builder.position_at_end(then_block);
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block)?;

    // "Panic" block : type is not the same
    cg.builder.position_at_end(panic_block);
    smolpp_panic(cg, RuntimeErrorMsg::TypeError, &[msg_str.into()])?;
    // End execution
    cg.builder.build_unreachable()?;

    // Merge block
    cg.builder.position_at_end(merge_block);

    return Ok(merge_block);
}


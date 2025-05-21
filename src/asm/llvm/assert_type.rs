use inkwell::{basic_block::BasicBlock, values::IntValue, AddressSpace, IntPredicate};

use crate::{
    asm::{
        codegen::CodeGen, get_internal_func, internal_global_constants::RuntimeErrorMsg,
        InternalFuctions,
    },
    common::localizable::Localizable,
    typing::Type,
};

use super::{panic::smolpp_panic_with_unreachable, smolvar::SmolVar, LLVMCodegenError};

/// Generate LLVM to assert the type of a variable at runtime
/// The runtime type must be exactly `valtype`, weak types will not match
/// if they have other possible types.
pub fn assert_type<'ctx, T>(
    valtype: Type,
    value: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
    msg: Option<String>,
    localizable: Option<T>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    let msg = match msg {
        Some(s) => s,
        None => format!("Expected type {}", valtype),
    };

    let type_field = cg.get_variable_type(*value)?;
    let expected_type = cg
        .context
        .i8_type()
        .const_int(valtype.get_bitmask() as u64, false);

    let cdt = cg.builder.build_int_compare(
        IntPredicate::EQ,
        expected_type,
        type_field,
        format!("assert_type_{}", valtype).as_str(),
    )?;

    return create_assert_type_branch(cdt, cg, msg, localizable, *value);
}

/// Generate LLVM to assert the type of a variable at runtime
/// The runtime type must be one of the types in `types`.
pub fn assert_type_oneof<'ctx, T>(
    types: &[Type],
    value: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
    msg: Option<String>,
    localizable: Option<T>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    let expected_bitmask: u8 = types
        .iter()
        .map(Type::get_bitmask)
        .reduce(|acc, bitmask| acc | bitmask)
        .expect("Cannot assert empty type");

    let msg = match msg {
        Some(s) => s,
        None => {
            let types_str: String = types
                .iter()
                .map(Type::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("Expected type {}", types_str)
        }
    };

    // _ separated list of accepted types
    let expected_types_str: String = types
        .iter()
        .map(Type::to_string)
        .collect::<Vec<_>>()
        .join("_");

    let type_field = cg.get_variable_type(*value)?;
    let expected_type = cg
        .context
        .i8_type()
        .const_int(expected_bitmask as u64, false);

    let and_res = cg.builder.build_and(
        expected_type,
        type_field,
        format!("assert_type_oneof_{}", expected_types_str).as_str(),
    )?;
    let cdt = cg.builder.build_int_compare(
        IntPredicate::NE,
        cg.context.i8_type().const_zero(),
        and_res,
        "convert_to_bool",
    )?;

    return create_assert_type_branch(cdt, cg, msg, localizable, *value);
}

/// Create the conditional branch for type assertion.
/// If `cdt` is true, the type check is considered successful.
/// If its false, the programs print an error message and exits.
/// Returns the basic block after the branch
fn create_assert_type_branch<'ctx, T>(
    cdt: IntValue<'ctx>,
    cg: &CodeGen<'ctx>,
    msg: String,
    localizable: Option<T>,
    value: SmolVar<'ctx>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    let string_value = cg.context.const_string(msg.as_str().as_bytes(), true);

    // Declare it as a global variable
    let global_var = cg.module.add_global(
        string_value.get_type(),
        Some(AddressSpace::default()),
        "__assert_type_msg_".into(),
    );

    global_var.set_initializer(&string_value);
    global_var.set_constant(true);

    // Create panic block and continuation block
    let ok_block = cg.context.append_basic_block(cg.current_function, "ok");
    let panic_block = cg.context.append_basic_block(cg.current_function, "panic");

    // Conditional branch
    cg.builder
        .build_conditional_branch(cdt, ok_block, panic_block)?;

    // Value
    let call_type_value = cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::Type),
        &[value.into()],
        "type_call",
    )?;

    // La fonction Type retourne directement un pointeur
    let actual_type_ptr = call_type_value.try_as_basic_value().unwrap_left();

    // "Panic" block : type is not the same
    cg.builder.position_at_end(panic_block);
    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::TypeError,
        &[global_var.as_pointer_value().into(), actual_type_ptr.into()],
        localizable,
    )?;

    // "Then" block : type is ok
    cg.builder.position_at_end(ok_block);

    return Ok(ok_block);
}

/// Generate LLVM to assert that the type of an expression is
/// compatible with its destination at runtime
/// This will stop the program if the runtime type of `value` is not
/// compatible with the runtime type of `destination`
pub fn assert_assignation_type<'ctx, T>(
    destination: &SmolVar<'ctx>,
    value: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
    localizable: Option<T>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    let value_type_field = cg.get_variable_type(*value)?;
    let dest_type_field = cg.get_variable_type(*destination)?;

    let and_res = cg
        .builder
        .build_and(dest_type_field, value_type_field, "assert_type_assign")?;
    let cdt = cg.builder.build_int_compare(
        IntPredicate::NE,
        cg.context.i8_type().const_zero(),
        and_res,
        "convert_to_bool",
    )?;

    return create_assert_type_branch(
        cdt,
        cg,
        String::from("Incompatible types during assignation"),
        localizable,
        *value,
    );
}

pub fn assert_dyn_type<'ctx, T>(
    value1: &SmolVar<'ctx>,
    value2: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
    localizable: Option<T>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    let t1 = cg.get_variable_type(*value1)?;
    let t2 = cg.get_variable_type(*value2)?;
    let cdt = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t1, t2, "assert_dyn_type")?;
    return create_assert_type_branch(
        cdt,
        cg,
        String::from("Runtime type mismatch in comparison"),
        localizable,
        *value1,
    );
}

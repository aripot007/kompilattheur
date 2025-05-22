use inkwell::{
    basic_block::BasicBlock,
    values::{GlobalValue, IntValue},
    AddressSpace, IntPredicate,
};

use crate::{
    asm::{
        codegen::CodeGen, get_internal_global_const, internal_global_constants::RuntimeErrorMsg,
        InternalGlobalConst,
    },
    common::localizable::Localizable,
    smollib::{get_smollib_func, SmollibFunctionNames},
    typing::Type,
};

use super::super::{panic::smolpp_panic_with_unreachable, smolvar::SmolVar, LLVMCodegenError};

/// Generate LLVM to assert the type of a variable at runtime
/// The runtime type must be exactly `valtype`, weak types will not match
/// if they have other possible types.
pub fn assert_type<'ctx, T>(
    valtype: Type,
    value: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
    localizable: Option<T>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    let global_const = match valtype {
        Type::None => InternalGlobalConst::ExpectedTypeNone,
        Type::Int => InternalGlobalConst::ExpectedTypeInt,
        Type::Bool => InternalGlobalConst::ExpectedTypeBool,
        Type::String => InternalGlobalConst::ExpectedTypeString,
        Type::List => InternalGlobalConst::ExpectedTypeList,
        Type::Range => InternalGlobalConst::ExpectedTypeRange,
        _ => panic!("Create a global constant for the type"),
    };
    let global_var = get_internal_global_const!(cg, global_const);

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

    // Get GlobalValue for the error message

    return create_assert_type_branch(cdt, cg, global_var, localizable, *value, None);
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

    let string_value = cg.context.const_string(msg.as_str().as_bytes(), true);

    // Declare it as a global variable
    let global_var = cg.module.add_global(
        string_value.get_type(),
        Some(AddressSpace::default()),
        "__assert_type_msg_".into(),
    );

    global_var.set_initializer(&string_value);
    global_var.set_constant(true);

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

    return create_assert_type_branch(cdt, cg, global_var, localizable, *value, None);
}

/// Create the conditional branch for type assertion.
/// If `cdt` is true, the type check is considered successful.
/// If its false, the programs print an error message and exits.
/// Returns the basic block after the branch
fn create_assert_type_branch<'ctx, T>(
    cdt: IntValue<'ctx>,
    cg: &CodeGen<'ctx>,
    global_var: GlobalValue<'ctx>,
    localizable: Option<T>,
    value: SmolVar<'ctx>,
    value2: Option<SmolVar<'ctx>>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    // Create panic block and continuation block
    let ok_block = cg.context.append_basic_block(cg.current_function, "ok");
    let panic_block = cg.context.append_basic_block(cg.current_function, "panic");

    // Conditional branch
    cg.builder
        .build_conditional_branch(cdt, ok_block, panic_block)?;

    // "Panic" block : type is not the same
    cg.builder.position_at_end(panic_block);

    // Value
    let call_type_value = cg.builder.build_call(
        get_smollib_func!(cg, SmollibFunctionNames::SmolType),
        &[value.into()],
        "type_call",
    )?;

    // La fonction Type retourne directement un pointeur
    let smol_var = call_type_value
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_struct_value();

    let smol_var_value = cg.get_variable_value(smol_var)?.into_int_value();

    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let smol_var_ptr = cg
        .builder
        .build_int_to_ptr(smol_var_value, ptr_type, "smol_var_ptr")?;

    let actual_type_ptr = cg.build_get_string_array_ptr_from_ptr(smol_var_ptr)?;

    if value2.is_some() {
        let value2 = value2.unwrap();
        let call_type_value2 = cg.builder.build_call(
            get_smollib_func!(cg, SmollibFunctionNames::SmolType),
            &[value2.into()],
            "type_call",
        )?;

        // La fonction Type retourne directement un pointeur
        let smol_var2 = call_type_value2
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_struct_value();

        let smol_var_value2 = cg.get_variable_value(smol_var2)?.into_int_value();

        let smol_var_ptr2 =
            cg.builder
                .build_int_to_ptr(smol_var_value2, ptr_type, "smol_var_ptr")?;

        let actual_type_ptr2 = cg.build_get_string_array_ptr_from_ptr(smol_var_ptr2)?;

        // Call the panic function with the two types
        smolpp_panic_with_unreachable(
            cg,
            RuntimeErrorMsg::TypeErrorDyn,
            &[
                global_var.as_pointer_value().into(),
                actual_type_ptr.into(),
                actual_type_ptr2.into(),
                actual_type_ptr.into(),
            ],
            localizable,
        )?;
    } else {
        smolpp_panic_with_unreachable(
            cg,
            RuntimeErrorMsg::TypeError,
            &[global_var.as_pointer_value().into(), actual_type_ptr.into()],
            localizable,
        )?;
    }

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

    let string_value = cg
        .context
        .const_string("Incompatible types during assignation".as_bytes(), true);

    // Declare it as a global variable
    let global_var = cg.module.add_global(
        string_value.get_type(),
        Some(AddressSpace::default()),
        "__assert_type_msg_".into(),
    );

    global_var.set_initializer(&string_value);
    global_var.set_constant(true);

    return create_assert_type_branch(cdt, cg, global_var, localizable, *value, None);
}

pub fn assert_dyn_type<'ctx, T>(
    value1: &SmolVar<'ctx>,
    value2: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
    msg: InternalGlobalConst,
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

    let global_var = get_internal_global_const!(cg, msg);
    return create_assert_type_branch(cdt, cg, global_var, localizable, *value1, Some(*value2));
    // String::from("Runtime type mismatch in comparison")
}

use inkwell::IntPredicate;

use crate::{
    asm::{
        codegen::CodeGen, internal_global_constants::RuntimeErrorMsg, llvm::LLVMCodegenError,
        InternalFuctions,
    },
    common::localizable::LocalizationInfo,
    smollib::{get_smollib_func, SmollibFunctionNames},
    typing::Type,
};

use super::panic::smolpp_panic_with_unreachable;

pub fn init_internal_bool_cast_function(cg: &mut CodeGen) -> Result<(), LLVMCodegenError> {
    // Create the function
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = cg
        .context
        .bool_type()
        .fn_type(&vec![var_type.into(); 1], false);

    let function = cg
        .module
        .add_function(InternalFuctions::BoolCast.into(), func_type, None);

    let entry = cg.context.append_basic_block(function, "bool_cast_entry");
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let var1 = function
        .get_nth_param(0 as u32)
        .unwrap()
        .into_struct_value();

    let t1 = cg.get_variable_type(var1)?;
    let var1_value = cg.get_variable_value(var1)?.into_int_value();
    // Switch
    // Si value1 == None => false en int1
    // Si Int / Bool => compare NE 0
    // Si String ou List => len(value1) NE 0
    let case_none = cg
        .context
        .append_basic_block(function, "bool_cast_case_none");
    let case_int_bool_range = cg
        .context
        .append_basic_block(function, "bool_cast_case_int_bool_range");
    let case_string_list = cg
        .context
        .append_basic_block(function, "bool_cast_case_string");
    let default_block = cg.context.append_basic_block(function, "bool_cast_default");

    let i8_type = cg.context.i8_type();

    cg.builder.build_switch(
        t1,
        default_block,
        &[
            (
                i8_type.const_int(Type::None.get_bitmask().into(), false),
                case_none,
            ),
            (
                i8_type.const_int(Type::Bool.get_bitmask().into(), false),
                case_int_bool_range,
            ),
            (
                i8_type.const_int(Type::Int.get_bitmask().into(), false),
                case_int_bool_range,
            ),
            (
                i8_type.const_int(Type::Range.get_bitmask().into(), false),
                case_int_bool_range,
            ),
            (
                i8_type.const_int(Type::String.get_bitmask().into(), false),
                case_string_list,
            ),
            (
                i8_type.const_int(Type::List.get_bitmask().into(), false),
                case_string_list,
            ),
        ],
    )?;

    cg.builder.position_at_end(case_none);
    // Si value1 == None => false en int1
    let false_value = cg.context.bool_type().const_zero();
    cg.builder.build_return(Some(&false_value))?;

    cg.builder.position_at_end(case_int_bool_range);
    // Si Int / Bool => compare NE 0
    let result = cg.builder.build_int_compare(
        IntPredicate::NE,
        var1_value,
        cg.context.i64_type().const_zero(),
        "bool_cast_int_bool",
    )?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(case_string_list);
    // Si String ou List => len(value1) NE 0
    let call_len_value = cg.builder.build_call(
        get_smollib_func!(cg, SmollibFunctionNames::SmolLen),
        &[var1.into()],
        "len_call",
    )?;

    let return_var = call_len_value
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_struct_value();

    let return_var_value = cg.get_variable_value(return_var)?.into_int_value();
    let result = cg.builder.build_int_compare(
        IntPredicate::NE,
        return_var_value,
        cg.context.i64_type().const_zero(),
        "bool_cast_string_list",
    )?;

    cg.builder.build_return(Some(&result))?;

    // Default case, print error message
    cg.builder.position_at_end(default_block);

    smolpp_panic_with_unreachable::<LocalizationInfo>(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
        &[t1.into()],
        None, //FIXME: potentially add localization info, not sure how to do that with internal functions
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    Ok(())
}

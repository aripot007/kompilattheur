use inkwell::AddressSpace;

use crate::{
    asm::{
        codegen::CodeGen,
        get_internal_func,
        llvm::{
            assert_type::{assert_dyn_type, assert_type},
            lists::llvm_build_list_concat,
            panic::smolpp_panic_with_unreachable,
            smolvar::SmolVar,
            strings::llvm_build_string_concat,
            LLVMCodegenError,
        },
        InternalFuctions, InternalGlobalConst, RuntimeErrorMsg,
    },
    common::localizable::{Localizable, LocalizationInfo},
    smollib::{get_smollib_func, SmollibFunctionNames},
    typing::Type,
};

pub fn compute_mult<'ctx, T>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
    loc1: Option<T>,
    loc2: Option<T>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    assert_type(Type::Int, &x, cg, loc1)?;
    assert_type(Type::Int, &y, cg, loc2)?;
    return compute_mult_unchecked(x, y, cg);
}

pub fn compute_mult_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_mul(x_val, y_val, "mult")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_div<'ctx, T>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
    loc1: Option<T>,
    loc2: Option<T>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    assert_type(Type::Int, &x, cg, loc1)?;
    assert_type(Type::Int, &y, cg, loc2)?;
    return compute_div_unchecked(x, y, cg);
}

pub fn compute_div_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_signed_div(x_val, y_val, "div")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_mod<'ctx, T>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
    loc1: Option<T>,
    loc2: Option<T>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    assert_type(Type::Int, &x, cg, loc1)?;
    assert_type(Type::Int, &y, cg, loc2)?;
    return compute_mod_unchecked(x, y, cg);
}

pub fn compute_mod_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_signed_rem(x_val, y_val, "mod")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_sub<'ctx, T>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
    loc1: Option<T>,
    loc2: Option<T>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    assert_type(Type::Int, &x, cg, loc1)?;
    assert_type(Type::Int, &y, cg, loc2)?;
    return compute_sub_unchecked(x, y, cg);
}

pub fn compute_sub_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_sub(x_val, y_val, "sub")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_add<'ctx, T>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
    loc1: Option<T>,
    loc2: Option<T>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    assert_type(Type::Int, &x, cg, loc1)?;
    assert_type(Type::Int, &y, cg, loc2)?;
    return compute_add_unchecked(x, y, cg);
}

pub fn compute_add_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_add(x_val, y_val, "add")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_add_range<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_add(x_val, y_val, "add")?;
    return cg.create_variable(Type::Range, res);
}

pub fn compute_add_list<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    return llvm_build_list_concat(x, y, cg);
}

pub fn compute_add_string<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    return llvm_build_string_concat(x, y, cg);
}

pub fn compute_add_generic<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let res = cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::GenericAdd),
        &[x.into(), y.into()],
        "generic_add_call",
    )?;
    return Ok(res.try_as_basic_value().left().unwrap().into_struct_value());
}

pub fn init_internal_add_generic_function<'ctx>(
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Create the function
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = var_type.fn_type(&vec![var_type.into(); 2], false);

    let function = cg
        .module
        .add_function(InternalFuctions::GenericAdd.into(), func_type, None);

    // Build the function
    let entry = cg
        .context
        .append_basic_block(function, "generic_add_function_entry");

    // Switch builder to the function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    // Get function parameter value
    let value1 = function
        .get_nth_param(0 as u32)
        .unwrap()
        .into_struct_value();
    let value2 = function
        .get_nth_param(1 as u32)
        .unwrap()
        .into_struct_value();

    // Assert they are the same type
    assert_dyn_type::<LocalizationInfo>(
        &value1,
        &value2,
        cg,
        InternalGlobalConst::CanOnlyConcatenate,
        None,
    )?; //FIXME: potentially add localization info, but i don't know how I am supposed to do that with generic functions

    // Load runtime type tags
    let typ = cg.get_variable_type(value1)?;

    let case_int = cg
        .context
        .append_basic_block(function, "generic_add_case_int");
    let case_range = cg
        .context
        .append_basic_block(function, "generic_add_case_range");
    let case_string = cg
        .context
        .append_basic_block(function, "generic_add_case_string");
    let case_list = cg
        .context
        .append_basic_block(function, "generic_add_case_list");
    let default_block = cg
        .context
        .append_basic_block(function, "generic_add_default");

    let i8_type = cg.context.i8_type();

    cg.builder.build_switch(
        typ,
        default_block,
        &[
            (
                i8_type.const_int(Type::Int.get_bitmask().into(), false),
                case_int,
            ),
            (
                i8_type.const_int(Type::Range.get_bitmask().into(), false),
                case_range,
            ),
            (
                i8_type.const_int(Type::String.get_bitmask().into(), false),
                case_string,
            ),
            (
                i8_type.const_int(Type::List.get_bitmask().into(), false),
                case_list,
            ),
        ],
    )?;

    cg.builder.position_at_end(case_int);
    let result = compute_add_unchecked(value1, value2, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(case_range);
    let result = compute_add_range(value1, value2, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(case_string);
    let result = compute_add_string(value1, value2, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(case_list);
    let result = compute_add_list(value1, value2, cg)?;
    cg.builder.build_return(Some(&result))?;

    // Default case, print error message
    cg.builder.position_at_end(default_block);

    let call_type_value = cg.builder.build_call(
        get_smollib_func!(cg, SmollibFunctionNames::SmolType),
        &[value1.into()],
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

    // Call the panic function with the two types
    smolpp_panic_with_unreachable::<LocalizationInfo>(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeAddGeneric,
        &[actual_type_ptr.into(), actual_type_ptr.into()],
        None,
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);
    return Ok(());
}

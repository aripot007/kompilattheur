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
        InternalFuctions, RuntimeErrorMsg,
    },
    typing::Type,
};

pub fn compute_mult<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
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

pub fn compute_div<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
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

pub fn compute_mod<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
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

pub fn compute_sub<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
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

pub fn compute_add<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
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
    assert_dyn_type(&value1, &value2, cg)?;

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

    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeAddGeneric,
        &[typ.into()],
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);
    return Ok(());
}

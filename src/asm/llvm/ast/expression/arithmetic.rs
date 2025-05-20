use crate::{
    asm::{
        codegen::CodeGen,
        get_internal_func,
        llvm::{
            assert_type, panic::smolpp_panic_with_unreachable, smolvar::SmolVar, LLVMCodegenError,
        },
        InternalFuctions, RuntimeErrorMsg,
    },
    typing::Type,
};
use inkwell::{
    values::{FunctionValue, IntValue},
    IntPredicate,
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
    // TODO: implement concat list
    let res = cg.context.i64_type().const_int(0, false);
    return cg.create_variable(Type::Int, res);
}

pub fn compute_add_string<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // TODO: implement concat string
    let res = cg.context.i64_type().const_int(0, false);
    return cg.create_variable(Type::Int, res);
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

    // Load runtime type tags
    let t1 = cg.get_variable_type(value1)?;
    let t2 = cg.get_variable_type(value2)?;

    // Check dynamique à l'execution Si c'est le même type, on fait la compare classique
    let same_type = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t1, t2, "dyn_eq")?;

    // Branch if the types are equal
    let parent_block = cg.builder.get_insert_block().unwrap();
    let then_block = cg
        .context
        .insert_basic_block_after(parent_block, "generic_add_same_type");
    let else_block = cg
        .context
        .insert_basic_block_after(then_block, "generic_add_different_type");
    let then_bool_or_int_block = cg
        .context
        .insert_basic_block_after(parent_block, "generic_add_same_type");
    let else_bool_or_int_block = cg
        .context
        .insert_basic_block_after(then_block, "generic_add_different_type");

    cg.builder
        .build_conditional_branch(same_type, then_block, else_block)?;

    // Case Same type
    cg.builder.position_at_end(then_block);

    build_switch_add_generic_same_type(cg, function, value1, value2, t1)?;

    // Case Different type
    cg.builder.position_at_end(else_block);

    // Create the comparaison calcul

    let i8_type = cg.context.i8_type();
    let bool_type = i8_type.const_int(Type::Bool.get_bitmask().into(), false);
    let int_type = i8_type.const_int(Type::Int.get_bitmask().into(), false);
    let mask = cg.builder.build_or(bool_type, int_type, "mask")?;
    let left_type = cg.builder.build_and(t1, mask, "left_type")?;
    let left_cond = cg.builder.build_int_compare(
        IntPredicate::NE,
        left_type,
        i8_type.const_zero(),
        "left_cond",
    )?;

    let right_type = cg.builder.build_and(t2, mask, "right_type")?;
    let right_cond = cg.builder.build_int_compare(
        IntPredicate::NE,
        right_type,
        i8_type.const_zero(),
        "right_cond",
    )?;

    let final_cond = cg.builder.build_and(left_cond, right_cond, "final_cond")?;

    // Si le type est Bool ou Int, alors on fait compare int
    cg.builder.build_conditional_branch(
        final_cond,
        then_bool_or_int_block,
        else_bool_or_int_block,
    )?;

    cg.builder.position_at_end(then_bool_or_int_block);
    let result = compute_add_unchecked(value1, value2, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(else_bool_or_int_block);

    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeAddGeneric,
        &[t1.into()],
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);
    return Ok(());
}

fn build_switch_add_generic_same_type<'ctx>(
    cg: &mut CodeGen<'ctx>,
    function: FunctionValue<'ctx>,
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    t1: IntValue<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Contruire un switch case dynamique en fonction du type de value1
    // Create a switch based on the type field

    let case_int_bool = cg
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
        t1,
        default_block,
        &[
            (
                i8_type.const_int(Type::Bool.get_bitmask().into(), false),
                case_int_bool,
            ),
            (
                i8_type.const_int(Type::Int.get_bitmask().into(), false),
                case_int_bool,
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

    cg.builder.position_at_end(case_int_bool);
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
        &[t1.into()],
    )?;

    return Ok(());
}

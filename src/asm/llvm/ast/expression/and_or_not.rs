use core::panic;

use inkwell::{AddressSpace, IntPredicate};

use crate::{
    asm::{
        codegen::CodeGen,
        get_internal_func,
        internal_global_constants::RuntimeErrorMsg,
        llvm::{
            ast::defs::user_function_prefix, panic::smolpp_panic_with_unreachable,
            smolvar::SmolVar, LLVMCodegenError,
        },
        InternalFuctions,
    },
    ast::nodes::{BinOp, Expression},
    typing::Type,
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
        "bool_cast_call",
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
    let case_int_bool = cg
        .context
        .append_basic_block(function, "bool_cast_case_int");
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
                case_int_bool,
            ),
            (
                i8_type.const_int(Type::Int.get_bitmask().into(), false),
                case_int_bool,
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

    cg.builder.position_at_end(case_int_bool);
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
        get_internal_func!(cg, InternalFuctions::Len),
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

    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
        &[t1.into()],
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    Ok(())
}

pub fn init_len_function(cg: &mut CodeGen) -> Result<(), LLVMCodegenError> {
    // Create the function
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = var_type.fn_type(&vec![var_type.into(); 1], false);

    let function = cg
        .module
        .add_function(user_function_prefix!("len"), func_type, None);

    let entry = cg.context.append_basic_block(function, "len_entry");
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let var1 = function
        .get_nth_param(0 as u32)
        .unwrap()
        .into_struct_value();

    let t1 = cg.get_variable_type(var1)?;

    // TODO: Assert type is list or string

    let var1_value = cg.get_variable_value(var1)?.into_int_value();
    // Get the length of the list or string
    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let var1_ptr = cg
        .builder
        .build_int_to_ptr(var1_value, ptr_type, "list_ptr")?;

    // Load the SmolList
    let var1_list = cg
        .builder
        .build_load(cg.smolpp_types.list_type, var1_ptr, "list")?
        .into_struct_value();

    let interator_value_len = cg.build_get_list_length(var1_list)?;

    let return_value = cg.create_variable(Type::Int, interator_value_len)?;
    cg.builder.build_return(Some(&return_value))?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);
    return Ok(());
}

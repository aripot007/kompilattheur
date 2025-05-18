use crate::{
    asm::{codegen::CodeGen, InternalFuctions},
    typing::Type,
};

use super::{assert_type, LLVMCodegenError};
use inkwell::IntPredicate;

// Initialize the internal "range" function that will generate a list of numbers from 0 to n-1
pub fn init_internal_range_function<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    let void_type = cg.context.void_type();
    let param_type = cg.smolpp_types.dynamic_type;

    // register function in module
    let func_type = void_type.fn_type(&[param_type.into()], false);
    let function = cg
        .module
        .add_function(InternalFuctions::Range.into(), func_type, None);

    // build function
    let entry = cg.context.append_basic_block(function, "range_entry");
    let loop_check = cg.context.append_basic_block(function, "range_check");
    let loop_body = cg.context.append_basic_block(function, "range_body");
    let loop_end = cg.context.append_basic_block(function, "range_end");

    // switch builder to function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let param = function.get_first_param().unwrap().into_struct_value();

    // Assert that the type is an integer
    assert_type(
        Type::Int,
        &param,
        cg,
        Some("Expected i32 type for range parameter".to_string()),
    )?;

    let upper_bound = param.get_field_at_index(0).unwrap().into_int_value();
    let (list_var, list_ptr) = cg.build_list_variable(upper_bound, false)?;

    // initialize loop counter
    let i_ptr = cg.builder.build_alloca(cg.context.i64_type(), "i_ptr")?;
    cg.builder
        .build_store(i_ptr, cg.context.i64_type().const_zero())?;
    cg.builder.build_unconditional_branch(loop_check)?;

    // check
    cg.builder.position_at_end(loop_check);
    let i_val = cg
        .builder
        .build_load(cg.context.i64_type(), i_ptr, "i_val")?
        .into_int_value();
    let cond = cg
        .builder
        .build_int_compare(IntPredicate::ULT, i_val, upper_bound, "range_cond")?;
    cg.builder
        .build_conditional_branch(cond, loop_body, loop_end)?;

    // body
    cg.builder.position_at_end(loop_body);
    let list_struct = cg
        .builder
        .build_load(cg.smolpp_types.list_type, list_ptr, "list_struct")?
        .into_struct_value();
    let array_ptr = cg.build_get_list_array_ptr(list_struct)?;
    let elem_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            array_ptr,
            &[i_val.into()],
            "elem_ptr",
        )
    }?;
    let elem_var = cg.create_variable(Type::Int, i_val)?;
    cg.builder.build_store(elem_ptr, elem_var)?;

    // update length
    let len = cg.build_get_list_length(list_struct)?;
    let new_len =
        cg.builder
            .build_int_add(len, cg.context.i64_type().const_int(1, false), "inc_len")?;
    let updated = cg
        .builder
        .build_insert_value(list_struct, new_len, 0, "set_list_len")?
        .into_struct_value();
    cg.builder.build_store(list_ptr, updated)?;
    // increment i
    let one = cg.context.i64_type().const_int(1, false);
    let next_i = cg.builder.build_int_add(i_val, one, "inc_i")?;
    cg.builder.build_store(i_ptr, next_i)?;
    cg.builder.build_unconditional_branch(loop_check)?;

    // end
    cg.builder.position_at_end(loop_end);
    cg.builder.build_return(None)?;

    // restore
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);
    Ok(())
}

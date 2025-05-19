use crate::{
    asm::{
        codegen::CodeGen,
        internal_functions::{internal_function_prefix, InternalFuctions},
        llvm::LLVMCodegenError,
    },
    typing::Type,
};
use inkwell::values::FunctionValue;

/// Initialize the internal list comparison function
pub fn init_internal_list_cmp_function<'ctx>(
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let i8_type = cg.context.i8_type();
    let var_type = cg.smolpp_types.list_type;

    // Register the function in the module
    let func_type = i8_type.fn_type(&[var_type.into(), var_type.into()], false);
    let function: FunctionValue<'ctx> =
        cg.module
            .add_function(InternalFuctions::ListCmp.into(), func_type, None);

    // Build the function
    let entry = cg
        .context
        .append_basic_block(function, internal_function_prefix!("list_cmp_entry"));

    // Switch builder to the function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    // Get function parameters
    let param1 = function.get_nth_param(0).unwrap();
    let param2 = function.get_nth_param(1).unwrap();

    // def list_cmp(l1, l2):
    //   for e in l1:
    //     if e < l2[i]: return -1
    //     if e > l2[i]: return 1
    //     if i > maxlen(l1,l2): break
    //   if len(l1) < len(l2) return -1
    //   if len(l1) > len(l2) return 1
    //   return 0

    // Load list structures
    let list1_struct = param1.into_struct_value();
    let list2_struct = param2.into_struct_value();

    // Get lengths
    let list1_len = cg.build_get_list_length(list1_struct)?;
    let list2_len = cg.build_get_list_length(list2_struct)?;

    // Get the minimum length
    let min_len_cmp = cg.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        list1_len,
        list2_len,
        "min_len_cmp",
    )?;
    let min_len = cg
        .builder
        .build_select(min_len_cmp, list1_len, list2_len, "min_len")?
        .into_int_value();

    // Allocate memory
    let var_ptr = cg.builder.build_alloca(
        cg.smolpp_types.dynamic_type,
        format!("alloca_loop_var_element").as_str(),
    )?;

    // Store initial value with correct type
    let val = cg.create_variable(Type::Any, cg.context.i64_type().const_zero())?;
    cg.builder.build_store(var_ptr, val)?;

    // Load the SmolList
    let iterator_list = param1.into_struct_value();

    // Get parent block
    let parent_block = cg.builder.get_insert_block().unwrap();

    // Create the loop
    let for_loop_block = cg
        .context
        .insert_basic_block_after(parent_block, "for_loop_block");
    let for_exit = cg
        .context
        .insert_basic_block_after(for_loop_block, "for_loop_exit");

    // Create the internal index variable
    let internal_index_int = cg
        .builder
        .build_alloca(cg.context.i64_type(), "internal_index")?;
    cg.builder
        .build_store(internal_index_int, cg.context.i64_type().const_zero())?;

    let interator_value_len = cg.build_get_list_length(iterator_list)?;

    // Compare if the len of the iterator is equal to 0
    let guard_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        cg.context.i64_type().const_zero(),
        interator_value_len,
        "guard_comparison",
    )?;

    cg.builder
        .build_conditional_branch(guard_comparison, for_exit, for_loop_block)?;

    cg.builder.position_at_end(for_loop_block);

    // Get the the current value of the iterator list

    let iterator_loop_ptr = cg.build_get_list_array_ptr(iterator_list)?;

    // Load the intenal index value
    let internal_index_int_load = cg
        .builder
        .build_load(
            cg.context.i64_type(),
            internal_index_int,
            "internal_index_load",
        )?
        .into_int_value();

    // Get iterator[i]
    let iterator_i_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            iterator_loop_ptr,
            &[internal_index_int_load],
            "list_i_ptr",
        )
    }?;

    let iterator_i =
        cg.builder
            .build_load(cg.smolpp_types.dynamic_type, iterator_i_ptr, "list_i")?;

    cg.builder.build_store(var_ptr, iterator_i)?;

    //
    // TODO: cmp here
    //

    // Increment the internal index variable
    let increment_one = cg.builder.build_int_add(
        internal_index_int_load,
        cg.context.i64_type().const_int(1, false),
        "increment_one",
    )?;

    cg.builder.build_store(internal_index_int, increment_one)?;

    // Compare the internal index with the iterator length
    let loop_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        increment_one,
        min_len,
        "loop_comparison",
    )?;

    cg.builder
        .build_conditional_branch(loop_comparison, for_loop_block, for_exit)?;

    cg.builder.position_at_end(for_exit);

    //
    // TODO: Test lenght
    //

    cg.builder
        .build_return(Some(&cg.context.i8_type().const_int(0, false)))?;

    // Return builder to main block
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}

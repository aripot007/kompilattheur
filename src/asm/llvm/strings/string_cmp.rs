use crate::asm::{codegen::CodeGen, internal_functions::InternalFuctions, llvm::LLVMCodegenError};
use inkwell::{values::FunctionValue, IntPredicate};

pub fn register_internal_str_cmp_function<'ctx>(cg: &mut CodeGen<'ctx>) -> FunctionValue<'ctx> {
    let i64_type = cg.context.i64_type();
    let var_type = cg.smolpp_types.string_type;

    // Register the function in the module
    let func_type = i64_type.fn_type(&[var_type.into(), var_type.into()], false);
    let function: FunctionValue<'ctx> =
        cg.module
            .add_function(InternalFuctions::StrCmp.into(), func_type, None);

    return function;
}

/// Initialize the internal list comparison function
pub fn init_internal_str_cmp_function<'ctx>(
    function: FunctionValue<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let entry = cg.context.append_basic_block(function, "str_cmp_entry");

    // Switch builder to the function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    // Get function parameters
    let param1 = function.get_nth_param(0).unwrap();
    let param2 = function.get_nth_param(1).unwrap();

    // def str_cmp(s1, s2) -> signed i64:
    //   for c in range(len(s1)):
    //     if i >= len(s2): break;
    //     diff = s1[i] - s2[i]
    //     if diff != 0: return int64(diff)
    //   return len(s1) - len(s2)
    //
    //   ret_val = phi
    //   cmp phi -> i8

    // Load list structures
    let str1_struct = param1.into_struct_value();

    // Get parent block
    let parent_block = cg.builder.get_insert_block().unwrap();

    // Create the loop
    let for_loop_block = cg
        .context
        .insert_basic_block_after(parent_block, "for_loop_block");
    let for_cmp_char = cg
        .context
        .insert_basic_block_after(for_loop_block, "for_cmp_char");
    let char_neq_block = cg
        .context
        .insert_basic_block_after(for_cmp_char, "char_neq_block");
    let char_eq_block = cg
        .context
        .insert_basic_block_after(char_neq_block, "char_eq_block");
    let for_exit = cg
        .context
        .insert_basic_block_after(char_eq_block, "for_loop_exit");

    // Create the internal index variable i
    let internal_index_int = cg
        .builder
        .build_alloca(cg.context.i64_type(), "internal_index")?;
    cg.builder
        .build_store(internal_index_int, cg.context.i64_type().const_zero())?;

    let iterator_len = cg.build_get_string_length(str1_struct)?;

    // Compare if the len of the iterator is equal to 0
    let guard_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        cg.context.i64_type().const_zero(),
        iterator_len,
        "guard_comparison",
    )?;

    cg.builder
        .build_conditional_branch(guard_comparison, for_exit, for_loop_block)?;

    cg.builder.position_at_end(for_loop_block);

    // Load the intenal index value
    let internal_index_int_load = cg
        .builder
        .build_load(
            cg.context.i64_type(),
            internal_index_int,
            "internal_index_load",
        )?
        .into_int_value();

    // Get the strings structs
    let str1_struct = param1.into_struct_value();
    let str2_struct = param2.into_struct_value();

    // if i >= len(s2): break;
    let str2_len = cg.build_get_string_length(str2_struct)?;

    let cmp = cg.builder.build_int_compare(
        IntPredicate::UGE,
        internal_index_int_load,
        str2_len,
        "break_cdt",
    )?;

    cg.builder
        .build_conditional_branch(cmp, for_exit, for_cmp_char)?;

    cg.builder.position_at_end(for_cmp_char);

    let i8_type = cg.context.i8_type();
    let i64_type = cg.context.i64_type();

    // diff = s1[i] - s2[i]
    let str1_array_ptr = cg.build_get_string_array_ptr(str1_struct)?;
    let str2_array_ptr = cg.build_get_string_array_ptr(str2_struct)?;

    let c1_ptr = unsafe {
        cg.builder.build_gep(
            i8_type,
            str1_array_ptr,
            &[internal_index_int_load],
            "s1_i_ptr",
        )
    }?;
    let c1_i8 = cg
        .builder
        .build_load(i8_type, c1_ptr, "s1_i")?
        .into_int_value();
    let c1 = cg.builder.build_int_cast(c1_i8, i64_type, "s1_i_i64")?;

    let c2_ptr = unsafe {
        cg.builder.build_gep(
            i8_type,
            str2_array_ptr,
            &[internal_index_int_load],
            "s2_i_ptr",
        )
    }?;
    let c2_i8 = cg
        .builder
        .build_load(i8_type, c2_ptr, "s2_i")?
        .into_int_value();
    let c2 = cg.builder.build_int_cast(c2_i8, i64_type, "s2_i_i64")?;

    let diff = cg.builder.build_int_sub(c1, c2, "diff")?;

    // if diff != 0: return int64(diff)

    let char_not_eq_cdt = cg.builder.build_int_compare(
        IntPredicate::NE,
        diff,
        i64_type.const_zero(),
        "char_not_eq",
    )?;

    cg.builder
        .build_conditional_branch(char_not_eq_cdt, char_neq_block, char_eq_block)?;

    cg.builder.position_at_end(char_neq_block);
    cg.builder.build_return(Some(&diff))?;

    cg.builder.position_at_end(char_eq_block);

    // Increment the internal index variable
    let increment_one = cg.builder.build_int_add(
        internal_index_int_load,
        cg.context.i64_type().const_int(1, false),
        "increment_one",
    )?;

    cg.builder.build_store(internal_index_int, increment_one)?;

    let str1_struct = param1.into_struct_value();
    let str1_len = cg.build_get_string_length(str1_struct)?;
    let loop_cdt =
        cg.builder
            .build_int_compare(IntPredicate::ULT, increment_one, str1_len, "loop_cdt")?;

    cg.builder
        .build_conditional_branch(loop_cdt, for_loop_block, for_exit)?;

    cg.builder.position_at_end(for_exit);

    let str1_struct = param1.into_struct_value();
    let str1_len = cg.build_get_string_length(str1_struct)?;
    let str2_struct = param2.into_struct_value();
    let str2_len = cg.build_get_string_length(str2_struct)?;

    let len_diff = cg.builder.build_int_sub(str1_len, str2_len, "len_diff")?;

    cg.builder.build_return(Some(&len_diff))?;

    // Return builder to main block
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}

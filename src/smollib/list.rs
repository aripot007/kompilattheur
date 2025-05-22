use inkwell::{
    types::FunctionType,
    values::{BasicValue, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{panic::smolpp_panic_with_unreachable, smolvar::SmolVar},
        LLVMCodegenError, RuntimeErrorMsg,
    },
    common::localizable::LocalizationInfo,
    typing::{Function, Type, Weak},
};

use super::{get_smollib_func, SmollibFunction, SmollibFunctionNames};

pub(super) struct SmolList {}

impl SmollibFunction for SmolList {
    fn name(&self) -> &str {
        "list"
    }

    fn func_type(&self) -> Function {
        let arg_type = Weak::new_with_possible(&[Type::List, Type::String, Type::Range]).locked();
        Function {
            args: vec![Type::Weak(arg_type)],
            returns: Type::List,
        }
    }

    fn llvm_type<'ctx>(&self, cg: &CodeGen<'ctx>) -> FunctionType<'ctx> {
        let var_type = cg.smolpp_types.dynamic_type;
        let func_type = var_type.fn_type(&vec![var_type.into(); 1], false);
        return func_type;
    }

    fn build_llvm<'ctx>(
        &self,
        function: FunctionValue<'ctx>,
        cg: &mut CodeGen<'ctx>,
    ) -> Result<(), LLVMCodegenError> {
        let entry = cg.context.append_basic_block(function, "list_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let var1 = function
            .get_nth_param(0 as u32)
            .unwrap()
            .into_struct_value();

        // Si type == Range => return Create the list with the range

        let t1 = cg.get_variable_type(var1)?;

        let case_range = cg
            .context
            .append_basic_block(function, "list_function_case_int");
        let case_string_or_list = cg
            .context
            .append_basic_block(function, "list_function_case_string_or_list");
        let default_block = cg
            .context
            .append_basic_block(function, "list_function_default");

        let i8_type = cg.context.i8_type();

        cg.builder.build_switch(
            t1,
            default_block,
            &[
                (
                    i8_type.const_int(Type::None.get_bitmask().into(), false),
                    default_block,
                ),
                (
                    i8_type.const_int(Type::Bool.get_bitmask().into(), false),
                    default_block,
                ),
                (
                    i8_type.const_int(Type::Int.get_bitmask().into(), false),
                    default_block,
                ),
                (
                    i8_type.const_int(Type::String.get_bitmask().into(), false),
                    case_string_or_list,
                ),
                (
                    i8_type.const_int(Type::List.get_bitmask().into(), false),
                    case_string_or_list,
                ),
                (
                    i8_type.const_int(Type::Range.get_bitmask().into(), false),
                    case_range,
                ),
            ],
        )?;

        cg.builder.position_at_end(case_range);
        let result = create_list_variable_from_range(cg, var1)?;
        cg.builder.build_return(Some(&result))?;

        cg.builder.position_at_end(case_string_or_list);

        // For List type => return the same value
        // For String type => convert to list of characters
        let type_string = cg
            .context
            .i8_type()
            .const_int(Type::String.get_bitmask().into(), false);
        let is_string = cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            t1,
            type_string,
            "is_string",
        )?;

        let case_string = cg.context.append_basic_block(function, "case_string");
        let case_list = cg.context.append_basic_block(function, "case_list");

        cg.builder
            .build_conditional_branch(is_string, case_string, case_list)?;

        // String case: convert to list of characters
        cg.builder.position_at_end(case_string);
        let result_string = create_list_from_string(cg, var1)?;
        cg.builder.build_return(Some(&result_string))?;

        // List case: return as is
        cg.builder.position_at_end(case_list);
        cg.builder.build_return(Some(&var1))?;

        // Default case, print error message
        cg.builder.position_at_end(default_block);

        let call_type_value = cg.builder.build_call(
            get_smollib_func!(cg, SmollibFunctionNames::SmolType),
            &[var1.into()],
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

        smolpp_panic_with_unreachable::<LocalizationInfo>(
            cg,
            RuntimeErrorMsg::InvalidTypeListFunction,
            &[actual_type_ptr.into()],
            None,
        )?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

fn create_list_variable_from_range<'ctx>(
    cg: &mut CodeGen<'ctx>,
    var1: SmolVar<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // TODO: Assert if var1 < i32 max
    let capa = cg.get_variable_value(var1)?.into_int_value();
    let (val, list_struct_ptr) = cg.build_list_variable(capa, true)?;

    // Update len
    let len_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        0,
        "create_list_from_range_len_ptr",
    )?;
    cg.builder.build_store(len_ptr, capa)?;

    let array_ptr_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        2,
        "array_ptr_ptr",
    )?;

    loop_to_create_list_from_range(cg, capa, array_ptr_ptr)?;

    return Ok(val);
}

fn loop_to_create_list_from_range<'ctx>(
    cg: &mut CodeGen<'ctx>,
    capa: IntValue<'ctx>,
    array_ptr_ptr: PointerValue<'ctx>,
) -> Result<(), LLVMCodegenError> {
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

    // Compare if the len of the iterator is equal to 0
    let guard_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        cg.context.i64_type().const_zero(),
        capa,
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

    // Cast the internal index to i32
    let list_index =
        cg.builder
            .build_int_cast(internal_index_int_load, cg.context.i32_type(), "list_index")?;

    let array_ptr = cg.builder.build_load(
        cg.context.ptr_type(AddressSpace::default()),
        array_ptr_ptr,
        "array_ptr",
    )?;

    // Load the intenal index value
    let elt_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            array_ptr.into_pointer_value(),
            &[list_index],
            "list_from_range_elt_ptr",
        )
    }?;

    let list_value =
        cg.create_variable(Type::Int, internal_index_int_load.as_basic_value_enum())?;

    cg.builder.build_store(elt_ptr, list_value)?;

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
        capa,
        "loop_comparison",
    )?;

    cg.builder
        .build_conditional_branch(loop_comparison, for_loop_block, for_exit)?;

    cg.builder.position_at_end(for_exit);

    return Ok(());
}

fn create_list_from_string<'ctx>(
    cg: &mut CodeGen<'ctx>,
    var1: SmolVar<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // Get the variable value as an int (pointer address)
    let ptr_int = cg.get_variable_value(var1)?.into_int_value();

    // Convert the int value to a pointer
    let ptr_type = cg.context.ptr_type(AddressSpace::default());
    let string_ptr = cg
        .builder
        .build_int_to_ptr(ptr_int, ptr_type, "string_ptr")?;

    // Load the string struct
    let string_struct = cg
        .builder
        .build_load(cg.smolpp_types.string_type, string_ptr, "string_struct")?
        .into_struct_value();

    // Get the length of the string
    let string_len = cg.build_get_string_length(string_struct)?;

    // Get the string data pointer
    let string_data = cg.build_get_string_array_ptr(string_struct)?;

    // Create a new list to hold the characters
    let (list_var, list_struct_ptr) = cg.build_list_variable(string_len, true)?;

    // Update the length of the list
    let list_len_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        0,
        "list_len_ptr",
    )?;
    cg.builder.build_store(list_len_ptr, string_len)?;

    // Get the array pointer where characters will be stored
    let list_array_ptr_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        2,
        "list_array_ptr_ptr",
    )?;

    // Fill the list with characters from the string
    fill_list_with_string_chars(cg, string_len, string_data, list_array_ptr_ptr)?;

    return Ok(list_var);
}

fn fill_list_with_string_chars<'ctx>(
    cg: &mut CodeGen<'ctx>,
    string_len: IntValue<'ctx>,
    string_data: PointerValue<'ctx>,
    list_array_ptr_ptr: PointerValue<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Get parent block
    let parent_block = cg.builder.get_insert_block().unwrap();

    // Create loop blocks
    let loop_block = cg
        .context
        .insert_basic_block_after(parent_block, "string_char_loop");
    let exit_block = cg
        .context
        .insert_basic_block_after(loop_block, "string_char_exit");

    // Create index variable
    let index_ptr = cg
        .builder
        .build_alloca(cg.context.i64_type(), "char_index")?;
    cg.builder
        .build_store(index_ptr, cg.context.i64_type().const_zero())?;

    // Check if string is empty
    let is_empty = cg.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        string_len,
        cg.context.i64_type().const_zero(),
        "string_is_empty",
    )?;

    cg.builder
        .build_conditional_branch(is_empty, exit_block, loop_block)?;

    // Loop body
    cg.builder.position_at_end(loop_block);

    // Load current index
    let current_index = cg
        .builder
        .build_load(cg.context.i64_type(), index_ptr, "current_index")?
        .into_int_value();

    // Convert to i32 for GEP
    let index_i32 = cg
        .builder
        .build_int_cast(current_index, cg.context.i32_type(), "index_i32")?;

    // Get character at current index
    let char_ptr = unsafe {
        cg.builder
            .build_gep(cg.context.i8_type(), string_data, &[index_i32], "char_ptr")
    }?;

    let char_value = cg
        .builder
        .build_load(cg.context.i8_type(), char_ptr, "char_value")?;

    // Create string for this single character (allocating on the heap)
    // We're using the buffer approach to create a single-character string
    let char_array = cg.builder.build_array_malloc(
        cg.context.i8_type(),
        cg.context.i64_type().const_int(2, false), // One character + null terminator
        "char_array",
    )?;

    // Store the character in the array
    cg.builder.build_store(char_array, char_value)?;

    // Store null terminator
    let null_ptr = unsafe {
        cg.builder.build_gep(
            cg.context.i8_type(),
            char_array,
            &[cg.context.i32_type().const_int(1, false)],
            "null_ptr",
        )
    }?;
    cg.builder
        .build_store(null_ptr, cg.context.i8_type().const_zero())?;

    // Create a string struct for the one character
    let one_len = cg.context.i64_type().const_int(1, false);
    let char_string_struct = cg.build_string_struct(one_len, char_array)?;

    // Allocate memory for the string struct in heap
    let char_string_ptr = cg
        .builder
        .build_malloc(cg.smolpp_types.string_type, "char_string_ptr")?;
    cg.builder
        .build_store(char_string_ptr, char_string_struct)?;

    // Convert the pointer to int for the SmolVar
    let ptr_int_type = cg
        .smolpp_types
        .dynamic_type
        .get_field_type_at_index(1)
        .unwrap()
        .into_int_type();
    let ptr_as_int = cg
        .builder
        .build_ptr_to_int(char_string_ptr, ptr_int_type, "ptr_as_int")?;

    // Create the SmolVar with the string pointer
    let char_var = cg.create_variable(Type::String, ptr_as_int)?;

    // Get list array pointer
    let list_array_ptr = cg.builder.build_load(
        cg.context.ptr_type(AddressSpace::default()),
        list_array_ptr_ptr,
        "list_array_ptr",
    )?;

    // Store character string in list
    let element_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            list_array_ptr.into_pointer_value(),
            &[index_i32],
            "element_ptr",
        )
    }?;

    cg.builder.build_store(element_ptr, char_var)?;

    // Increment index
    let next_index = cg.builder.build_int_add(
        current_index,
        cg.context.i64_type().const_int(1, false),
        "next_index",
    )?;

    cg.builder.build_store(index_ptr, next_index)?;

    // Check if we've processed all characters
    let continue_loop = cg.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        next_index,
        string_len,
        "continue_loop",
    )?;

    cg.builder
        .build_conditional_branch(continue_loop, loop_block, exit_block)?;

    // Exit block
    cg.builder.position_at_end(exit_block);

    return Ok(());
}

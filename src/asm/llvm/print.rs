use inkwell::{basic_block::BasicBlock, values::BasicValue, AddressSpace, IntPredicate};

use crate::{
    asm::{
        codegen::CodeGen, get_internal_func, get_internal_global_const, internal_function_prefix,
        internal_global_constants::RuntimeErrorMsg, InternalFuctions, InternalGlobalConst,
    },
    typing::Type,
};

use super::{panic::smolpp_panic_with_unreachable, smolvar::SmolVar, LLVMCodegenError};

macro_rules! llvm_printf {
    ($cg: expr, $value: expr, $name: literal) => {
        $cg.builder.build_call(
            $cg.module
                .get_function(InternalFuctions::Printf.into())
                .unwrap(),
            &[$value.into()],
            $name,
        )?;
    };
}

/// Generate LLVM to print a None value
pub fn print_none_value<'ctx>(
    _value: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let none_str_ptr =
        get_internal_global_const!(cg, InternalGlobalConst::NoneString).as_pointer_value();
    llvm_printf!(cg, none_str_ptr, "printf_none");
    return Ok(());
}

/// Generate LLVM to print a int value
pub fn print_int_value<'ctx>(
    variable: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Get the variable value as int
    let value = cg.get_variable_value(*variable)?.into_int_value();

    // Call printf("%d", value)
    cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::Printf),
        &[
            // Format string
            get_internal_global_const!(cg, InternalGlobalConst::IntFormatString)
                .as_pointer_value()
                .into(),
            value.into(), // value
        ],
        "printf_int",
    )?;
    return Ok(());
}

/// Generate LLVM to print a String value
pub fn print_string_value<'ctx>(
    variable: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let value_generic = cg.get_variable_value(*variable)?;

    let value = match value_generic.is_int_value() {
        true => cg
            .builder
            .build_int_to_ptr(value_generic.into_int_value(), ptr_type, "str_ptr")?,
        false => value_generic.into_pointer_value(),
    };

    llvm_printf!(cg, value, "printf_string");

    return Ok(());
}

/// Generate LLVM to print a Bool value
pub fn print_bool_value<'ctx>(
    variable: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let value = cg.get_variable_value(*variable)?.into_int_value();
    let bool_value = cg
        .builder
        .build_int_cast(value, cg.context.bool_type(), "value_as_bool")?;

    let false_val = cg.context.bool_type().const_zero();
    let cdt =
        cg.builder
            .build_int_compare(inkwell::IntPredicate::NE, bool_value, false_val, "cmp")?;

    // Create basic blocks if-else
    let then_block = cg.context.append_basic_block(cg.current_function, "then");
    let else_block = cg.context.append_basic_block(cg.current_function, "else");
    let merge_block = cg.context.append_basic_block(cg.current_function, "end");

    // Conditional branch
    cg.builder
        .build_conditional_branch(cdt, then_block, else_block)?;

    // "Then" block : value is True
    cg.builder.position_at_end(then_block);
    llvm_printf!(
        cg,
        get_internal_global_const!(cg, InternalGlobalConst::TrueString).as_pointer_value(),
        "printf_bool_true"
    );
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block)?;

    // "Else" block : value is False
    cg.builder.position_at_end(else_block);
    llvm_printf!(
        cg,
        get_internal_global_const!(cg, InternalGlobalConst::FalseString).as_pointer_value(),
        "printf_bool_false"
    );
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block)?;

    // Merge block
    cg.builder.position_at_end(merge_block);

    return Ok(());
}

/// Generate LLVM to print a List value
pub fn print_list_value<'ctx>(
    variable: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
) -> Result<BasicBlock<'ctx>, LLVMCodegenError> {
    // TODO : Escape strings

    let opening_str =
        get_internal_global_const!(cg, InternalGlobalConst::ListOpeningStr).as_pointer_value();
    let closing_str =
        get_internal_global_const!(cg, InternalGlobalConst::ListClosingStr).as_pointer_value();
    let separator =
        get_internal_global_const!(cg, InternalGlobalConst::ListSeparatorStr).as_pointer_value();

    let printf = cg
        .module
        .get_function(InternalFuctions::Printf.into())
        .unwrap();

    let ptr_type = cg.context.ptr_type(AddressSpace::default());
    let list_ptr = cg.get_variable_value(*variable)?;
    let list_ptr = cg
        .builder
        .build_int_to_ptr(list_ptr.into_int_value(), ptr_type, "list_ptr")?;
    let list = cg
        .builder
        .build_load(cg.smolpp_types.list_type, list_ptr, "list")?
        .into_struct_value();

    cg.builder
        .build_call(printf, &[opening_str.into()], "print_opening_brace")?;

    let current_block = cg
        .builder
        .get_insert_block()
        .expect("Builder is not in a block");
    let loop_header = cg
        .context
        .insert_basic_block_after(current_block, "list_print_loop_header");
    let loop_block = cg
        .context
        .insert_basic_block_after(loop_header, "list_print_loop");
    let merge_block = cg
        .context
        .insert_basic_block_after(loop_block, "list_print_loop_merge");
    let loop_end = cg
        .context
        .insert_basic_block_after(merge_block, "list_print_loop_end");

    let len = cg.build_get_list_length(list)?;

    // if (len == 0) goto loop_end;
    let empty_cdt = cg.builder.build_int_compare(
        IntPredicate::EQ,
        len,
        cg.context.i64_type().const_zero(),
        "empty_list_cdt",
    )?;
    cg.builder
        .build_conditional_branch(empty_cdt, loop_end, loop_header)?;

    cg.builder.position_at_end(loop_header);

    let array_ptr = cg.build_get_list_array_ptr(list)?;

    // int i = 0
    // loop: {
    //   print(list[i])
    //   i++;
    //   if (i >= len) goto loop_end;
    //   merge:
    //   print(", ")
    // }
    // loop_end:

    let i_ptr = cg
        .builder
        .build_alloca(cg.context.i64_type(), "list_print_loop_counter")?;
    cg.builder
        .build_store(i_ptr, cg.context.i64_type().const_zero())?;

    cg.builder.build_unconditional_branch(loop_block)?;
    cg.builder.position_at_end(loop_block);

    // Load i value
    let i = cg
        .builder
        .build_load(cg.context.i64_type(), i_ptr, "i.val")?
        .into_int_value();

    // Get list[i]
    let list_i_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            array_ptr,
            &[i.into()],
            "list_i_ptr",
        )
    }?;
    let list_i = cg
        .builder
        .build_load(cg.smolpp_types.dynamic_type, list_i_ptr, "list_i")?;

    // Print element
    print_any_value(&list_i.into_struct_value(), cg)?;

    // i++
    let one = cg.context.i64_type().const_int(1, false);
    let i = cg.builder.build_int_add(i, one, "increase")?;
    cg.builder.build_store(i_ptr, i)?;

    // if (i >= len) goto loop_end;
    let cmp = cg
        .builder
        .build_int_compare(IntPredicate::UGE, i, len, "list_print_loop_cond")?;
    cg.builder
        .build_conditional_branch(cmp, loop_end, merge_block)?;

    // print(", ")
    cg.builder.position_at_end(merge_block);
    cg.builder
        .build_call(printf, &[separator.into()], "print_separator")?;
    cg.builder.build_unconditional_branch(loop_block)?;

    // loop_end:
    cg.builder.position_at_end(loop_end);
    cg.builder
        .build_call(printf, &[closing_str.into()], "print_closing_brace")?;

    return Ok(loop_end);
}

/// Generate LLVM to print any value.
pub fn print_any_value<'ctx>(
    value: &SmolVar<'ctx>,
    cg: &CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::GenericPrint),
        &[value.as_basic_value_enum().into()],
        "generic_print_call",
    )?;
    return Ok(());
}

/// Initialize the internal "generic_print" function that can print any value
pub fn init_internal_generic_print_function<'ctx>(
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let void_type = cg.context.void_type();
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = void_type.fn_type(&[var_type.into()], false);
    let function = cg
        .module
        .add_function(InternalFuctions::GenericPrint.into(), func_type, None);

    // Build the function
    let entry = cg
        .context
        .append_basic_block(function, internal_function_prefix!("generic_print_entry"));

    // Switch builder to the function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let param = function.get_first_param().unwrap().into_struct_value();

    let type_field = cg.get_variable_type(param)?;

    // Create a switch based on the type field
    let case_none = cg.context.append_basic_block(function, "case_none");
    let case_bool = cg.context.append_basic_block(function, "case_bool");
    let case_int = cg.context.append_basic_block(function, "case_int");
    let case_string = cg.context.append_basic_block(function, "case_string");
    let case_list = cg.context.append_basic_block(function, "case_list");
    let default_block = cg.context.append_basic_block(function, "default");

    let i8_type = cg.context.i8_type();

    cg.builder.build_switch(
        type_field,
        default_block,
        &[
            (
                i8_type.const_int(Type::None.get_bitmask().into(), false),
                case_none,
            ),
            (
                i8_type.const_int(Type::Bool.get_bitmask().into(), false),
                case_bool,
            ),
            (
                i8_type.const_int(Type::Int.get_bitmask().into(), false),
                case_int,
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

    cg.builder.position_at_end(case_none);
    print_none_value(&param, cg)?;
    cg.builder.build_return(None)?;

    cg.builder.position_at_end(case_bool);
    print_bool_value(&param, cg)?;
    cg.builder.build_return(None)?;

    cg.builder.position_at_end(case_int);
    print_int_value(&param, cg)?;
    cg.builder.build_return(None)?;

    cg.builder.position_at_end(case_string);
    print_string_value(&param, cg)?;
    cg.builder.build_return(None)?;

    cg.builder.position_at_end(case_list);
    print_list_value(&param, cg)?;
    cg.builder.build_return(None)?;

    // Default case, print error message
    cg.builder.position_at_end(default_block);

    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeValueFormatString,
        &[type_field.into()],
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}

use inkwell::{values::{BasicValue, StructValue}, AddressSpace};

use crate::{asm::{codegen::CodeGen, get_internal_func, get_internal_global_const, internal_function_prefix, internal_global_constants::RuntimeErrorMsg, InternalFuctions, InternalGlobalConst}, typing::Type};

use super::{panic::{smolpp_panic, smolpp_panic_with_unreachable}, LLVMCodegenError};

macro_rules! llvm_puts {
    ($cg: expr, $value: expr, $name: literal) => {
        $cg.builder
            .build_call(
                $cg.module.get_function(InternalFuctions::Puts.into()).unwrap(),
                &[$value.into()], $name)?;
    };
}

/// Generate LLVM to print a None value
pub fn print_none_value<'ctx>(_value: &StructValue<'ctx>, cg: &CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    let none_str_ptr = get_internal_global_const!(cg, InternalGlobalConst::NoneString).as_pointer_value();
    llvm_puts!(cg, none_str_ptr, "puts_none");
    return Ok(());
}

/// Generate LLVM to print a int value
pub fn print_int_value<'ctx>(variable: &StructValue<'ctx>, cg: &CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    // Get the variable value as int
    let value = cg.builder.build_extract_value(*variable, 1, "value_field")?.into_int_value();

    // Call printf("%d", value)
    cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::Printf),
        &[
            // Format string
            get_internal_global_const!(cg, InternalGlobalConst::IntFormatStringWithNewline).as_pointer_value().into(),
            value.into() // value
        ],
        "printf_int"
    )?;
    return Ok(());
}

/// Generate LLVM to print a String value
pub fn print_string_value<'ctx>(variable: &StructValue<'ctx>, cg: &CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let value_generic = cg.builder.build_extract_value(*variable, 1, "value_field")?;

    let value = match value_generic.is_int_value() {
        true => cg.builder.build_int_to_ptr(value_generic.into_int_value(), ptr_type, "str_ptr")?,
        false => value_generic.into_pointer_value(),
    };

    llvm_puts!(cg, value, "puts_string");

    return Ok(());
}

/// Generate LLVM to print a Bool value
pub fn print_bool_value<'ctx>(variable: &StructValue<'ctx>, cg: &CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    let value = cg.builder.build_extract_value(*variable, 1, "value_field")?.into_int_value();

    let zero = cg.context.i64_type().const_zero();
    let cdt = cg.builder.build_int_compare(inkwell::IntPredicate::EQ, value, zero, "cmp")?;

    // Create basic blocks if-else
    let then_block = cg.context.append_basic_block(cg.current_function, "then");
    let else_block = cg.context.append_basic_block(cg.current_function, "else");
    let merge_block = cg.context.append_basic_block(cg.current_function, "end");

    // Conditional branch
    cg.builder.build_conditional_branch(cdt, then_block, else_block)?;

    // "Then" block : value is True
    cg.builder.position_at_end(then_block);
    llvm_puts!(cg, get_internal_global_const!(cg, InternalGlobalConst::TrueString).as_pointer_value(), "puts_bool_true");
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block)?;


    // "Else" block : value is False
    cg.builder.position_at_end(else_block);
    llvm_puts!(cg, get_internal_global_const!(cg, InternalGlobalConst::FalseString).as_pointer_value(), "puts_bool_false");
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block)?;

    // Merge block
    cg.builder.position_at_end(merge_block);

    return Ok(());

}

/// Generate LLVM to print a List value
pub fn print_list_value<'ctx>(value: &StructValue<'ctx>, cg: &CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    smolpp_panic(cg, RuntimeErrorMsg::PanicNotImplemented, &[])?;
    return Ok(());
}

/// Generate LLVM to print any value.
pub fn print_any_value<'ctx>(value: &StructValue<'ctx>, cg: &CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    cg.builder.build_call(get_internal_func!(cg, InternalFuctions::GenericPrint), &[value.as_basic_value_enum().into()], "generic_print_call")?;
    return Ok(());
}

/// Initialize the internal "generic_print" function that can print any value
pub fn init_internal_generic_print_function<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {

    let void_type = cg.context.void_type();
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = void_type.fn_type(&[var_type.into()], false);
    let function = cg.module
        .add_function(InternalFuctions::GenericPrint.into(), func_type, None);

    // Build the function
    let entry = cg.context.append_basic_block(function, internal_function_prefix!("generic_print_entry"));
    
    // Switch builder to the function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let param = function.get_first_param().unwrap().into_struct_value();

    let type_field = cg.builder.build_extract_value(param, 0, "type_field")?.into_int_value();

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
            (i8_type.const_int(Type::None.get_bitmask().into(), false), case_none),
            (i8_type.const_int(Type::Bool.get_bitmask().into(), false), case_bool),
            (i8_type.const_int(Type::Int.get_bitmask().into(), false), case_int),
            (i8_type.const_int(Type::String.get_bitmask().into(), false), case_string),
            (i8_type.const_int(Type::List.get_bitmask().into(), false), case_list),
        ]
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

    smolpp_panic_with_unreachable(cg, RuntimeErrorMsg::PanicInvalidInternalTypeValueFormatString, &[type_field.into()])?;

    // Return builder to main block
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}
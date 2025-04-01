use inkwell::values::StructValue;

use crate::asm::{codegen::CodeGen, get_internal_func, get_internal_global_const, InternalFuctions, InternalGlobalConst};

macro_rules! llvm_puts {
    ($cg: expr, $value: expr) => {
        $cg.builder
            .build_call(
                $cg.module.get_function(InternalFuctions::Puts.into()).unwrap(),
                &[$value.into()], "puts_call")
            .unwrap();
    };
}

/// Generate LLVM to print a None value
pub fn print_none_value<'ctx>(_value: &StructValue<'ctx>, cg: &CodeGen<'ctx>) {
    let none_str_ptr = get_internal_global_const!(cg, InternalGlobalConst::NoneString).as_pointer_value();
    llvm_puts!(cg, none_str_ptr);
}

/// Generate LLVM to print a int value
pub fn print_int_value<'ctx>(variable: &StructValue<'ctx>, cg: &CodeGen<'ctx>) {

    // Get the variable value as int
    let value = variable.get_field_at_index(1).unwrap().into_int_value();

    // Call printf("%d", value)
    cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::Printf),
        &[
            // Format string
            get_internal_global_const!(cg, InternalGlobalConst::IntFormatStringWithNewline).as_pointer_value().into(),
            value.into() // value
        ],
        "printf_call"
    ).unwrap();

}

/// Generate LLVM to print a String value
pub fn print_string_value<'ctx>(value: &StructValue<'ctx>, cg: &CodeGen<'ctx>) {
    llvm_puts!(cg, value.get_field_at_index(1).unwrap());
}

/// Generate LLVM to print a Bool value
pub fn print_bool_value<'ctx>(variable: &StructValue<'ctx>, cg: &CodeGen<'ctx>) {

    // Get the variable value as int
    let value = variable.get_field_at_index(1).unwrap().into_int_value();

    let zero = cg.context.i64_type().const_zero();
    let cdt = cg.builder.build_int_compare(inkwell::IntPredicate::EQ, value, zero, "cmp").unwrap();

    // Create basic blocks if-else
    let then_block = cg.context.append_basic_block(cg.current_function, "then");
    let else_block = cg.context.append_basic_block(cg.current_function, "else");
    let merge_block = cg.context.append_basic_block(cg.current_function, "end");

    // Conditional branch
    cg.builder.build_conditional_branch(cdt, then_block, else_block).unwrap();

    // "Then" block : value is True
    cg.builder.position_at_end(then_block);
    llvm_puts!(cg, get_internal_global_const!(cg, InternalGlobalConst::TrueString).as_pointer_value());
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block).unwrap();


    // "Else" block : value is False
    cg.builder.position_at_end(else_block);
    llvm_puts!(cg, get_internal_global_const!(cg, InternalGlobalConst::FalseString).as_pointer_value());
    // Go back to the main execution
    cg.builder.build_unconditional_branch(merge_block).unwrap();

    // Merge block
    cg.builder.position_at_end(merge_block);

}

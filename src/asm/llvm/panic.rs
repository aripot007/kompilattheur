use inkwell::{builder::BuilderError, values::BasicMetadataValueEnum};

use crate::asm::{codegen::CodeGen, get_internal_func, get_internal_global_const, internal_global_constants::RuntimeErrorMsg, InternalFuctions};

/// Generate llvm to exit the program after printing an error message
pub fn smolpp_panic<'ctx>(cg: &CodeGen<'ctx>, error: RuntimeErrorMsg, fmt_args: &[BasicMetadataValueEnum<'ctx>]) -> Result<(), BuilderError> {

    let fmt_string = get_internal_global_const!(cg, error).as_pointer_value().into();
    let args = &[&[fmt_string], fmt_args].concat();

    cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::Printf),
        args,
        "panic_printf"
    )?;

    // Crash the program
    let trap_func = get_internal_func!(cg, InternalFuctions::Trap);
    cg.builder.build_call(trap_func, &[], "call_trap")?;
    return Ok(());
}

/// Generate llvm to exit the program after printing an error message, an add an unreachable after the trap call.
/// Use this if LLVM reports that a block containing a panic does not have a terminator.
/// If LLVM reports that there is a terminator in the middleof a basic bloc, use `smolpp_panic` instead
pub fn smolpp_panic_with_unreachable<'ctx>(cg: &CodeGen<'ctx>, error: RuntimeErrorMsg, fmt_args: &[BasicMetadataValueEnum<'ctx>]) -> Result<(), BuilderError> {
    smolpp_panic(cg, error, fmt_args)?;
    cg.builder.build_unreachable()?;
    return Ok(());
}
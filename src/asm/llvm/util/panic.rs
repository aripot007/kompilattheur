use inkwell::{builder::BuilderError, values::BasicMetadataValueEnum};

use crate::{
    asm::{
        codegen::CodeGen, get_internal_func, get_internal_global_const,
        internal_global_constants::RuntimeErrorMsg, InternalFuctions,
    },
    common::localizable::Localizable,
};

/// Generate llvm to exit the program after printing an error message
pub fn smolpp_panic<'ctx, T>(
    cg: &CodeGen<'ctx>,
    error: RuntimeErrorMsg,
    fmt_args: &[BasicMetadataValueEnum<'ctx>],
    localizable: Option<T>,
) -> Result<(), BuilderError>
where
    T: Localizable,
{
    let fmt_string = get_internal_global_const!(cg, error)
        .as_pointer_value()
        .into();
    let args = &[&[fmt_string], fmt_args].concat();

    cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::Printf),
        args,
        "panic_printf",
    )?;

    // Print the line and column if they are available
    if let Some(localizable) = localizable {
        let fmt_string: BasicMetadataValueEnum<'ctx> =
            get_internal_global_const!(cg, RuntimeErrorMsg::LocalizeError)
                .as_pointer_value()
                .into();
        let line = localizable.get_start_line();
        let int_value_line = cg.context.i64_type().const_int(line as u64, false);
        let col = localizable.get_start_char();
        let int_value_col = cg.context.i64_type().const_int(col as u64, false);
        let args: &[BasicMetadataValueEnum<'_>; 3] = &[
            fmt_string.into(),
            int_value_line.into(),
            int_value_col.into(),
        ];

        cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::Printf),
            args,
            "panic_line_printf",
        )?;
    }

    // Crash the program
    let trap_func = get_internal_func!(cg, InternalFuctions::Trap);
    cg.builder.build_call(trap_func, &[], "call_trap")?;
    return Ok(());
}

/// Generate llvm to exit the program after printing an error message, an add an unreachable after the trap call.
/// Use this if LLVM reports that a block containing a panic does not have a terminator.
/// If LLVM reports that there is a terminator in the middleof a basic bloc, use `smolpp_panic` instead
pub fn smolpp_panic_with_unreachable<'ctx, T>(
    cg: &CodeGen<'ctx>,
    error: RuntimeErrorMsg,
    fmt_args: &[BasicMetadataValueEnum<'ctx>],
    localizable: Option<T>,
) -> Result<(), BuilderError>
where
    T: Localizable,
{
    smolpp_panic(cg, error, fmt_args, localizable)?;
    cg.builder.build_unreachable()?;
    return Ok(());
}

use inkwell::{builder::BuilderError, values::BasicMetadataValueEnum, AddressSpace};

use crate::{
    asm::{
        codegen::CodeGen, get_internal_func, get_internal_global_const, internal_function_prefix,
        internal_global_constants::RuntimeErrorMsg, InternalFuctions, InternalGlobalConst,
    },
    common::localizable::{Localizable, LocalizationInfo},
    typing::Type,
};

use super::LLVMCodegenError;

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

pub fn init_internal_type_function(cg: &mut CodeGen) -> Result<(), LLVMCodegenError> {
    let ptr_type = cg.context.ptr_type(AddressSpace::default());
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = ptr_type.fn_type(&[var_type.into()], false);
    let function = cg
        .module
        .add_function(InternalFuctions::Type.into(), func_type, None);

    // Build the function
    let entry = cg
        .context
        .append_basic_block(function, internal_function_prefix!("type_entry"));

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
    let case_range = cg.context.append_basic_block(function, "case_range");
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
            (
                i8_type.const_int(Type::Range.get_bitmask().into(), false),
                case_range,
            ),
        ],
    )?;

    cg.builder.position_at_end(case_none);
    let fmt_string =
        get_internal_global_const!(cg, InternalGlobalConst::NoneType).as_pointer_value();
    cg.builder.build_return(Some(&fmt_string))?;

    cg.builder.position_at_end(case_bool);
    let fmt_string =
        get_internal_global_const!(cg, InternalGlobalConst::BoolType).as_pointer_value();
    cg.builder.build_return(Some(&fmt_string))?;

    cg.builder.position_at_end(case_int);
    let fmt_string =
        get_internal_global_const!(cg, InternalGlobalConst::IntType).as_pointer_value();
    cg.builder.build_return(Some(&fmt_string))?;

    cg.builder.position_at_end(case_string);
    let fmt_string =
        get_internal_global_const!(cg, InternalGlobalConst::StringType).as_pointer_value();
    cg.builder.build_return(Some(&fmt_string))?;

    cg.builder.position_at_end(case_list);
    let fmt_string =
        get_internal_global_const!(cg, InternalGlobalConst::ListType).as_pointer_value();
    cg.builder.build_return(Some(&fmt_string))?;

    cg.builder.position_at_end(case_range);
    let fmt_string =
        get_internal_global_const!(cg, InternalGlobalConst::RangeType).as_pointer_value();
    cg.builder.build_return(Some(&fmt_string))?;

    // Default case, print error message
    cg.builder.position_at_end(default_block);

    smolpp_panic_with_unreachable::<LocalizationInfo>(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeInTypeFunction,
        &[type_field.into()],
        None, //FIXME: potentially add localization info, not sure how to do that with internal functions
    )?;

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}

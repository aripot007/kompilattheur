use crate::{
    ast::nodes::BinOp,
    smollib::{build_smollib_llvm, register_smollib_funcs_in_module},
};

use super::{
    codegen::CodeGen,
    llvm::{
        init_internal_add_generic_function, init_internal_bool_cast_function,
        init_internal_compare_generic_function, init_internal_generic_print_function,
        init_internal_list_cmp_function, pre_init_internal_list_cmp_function,
        strings::{init_internal_str_cmp_function, register_internal_str_cmp_function},
        LLVMCodegenError,
    },
};

pub enum InternalFuctions {
    Main,
    GenericPrint,
    GenericCompareEQ,
    GenericCompareNEQ,
    GenericCompareLESS,
    GenericCompareLESSEQ,
    GenericCompareGREATER,
    GenericCompareGREATEREQ,
    ListCmp,
    StrCmp,
    BoolCast,
    GenericAdd,
    // Syscalls
    Trap,
    Puts,
    Printf,
}

/// Get an internal function registered in the CodeGen.
/// The internal functions MUST be initialized before using this macro
macro_rules! get_internal_func {
    ($cg: expr, $name: expr) => {
        $cg.module.get_function($name.into()).unwrap()
    };
}
pub(super) use get_internal_func;

macro_rules! internal_function_prefix {
    ($name: expr) => {
        concat!("__smolpp_f_", $name)
    };
}
pub(crate) use internal_function_prefix;

impl Into<&'static str> for InternalFuctions {
    fn into(self) -> &'static str {
        match self {
            InternalFuctions::Main => "main",
            InternalFuctions::GenericPrint => internal_function_prefix!("generic_print"),
            InternalFuctions::ListCmp => internal_function_prefix!("list_cmp"),
            InternalFuctions::StrCmp => internal_function_prefix!("str_cmp"),
            InternalFuctions::Puts => "puts",
            InternalFuctions::Printf => "printf",
            InternalFuctions::Trap => "llvm.debugtrap",
            InternalFuctions::GenericCompareEQ => internal_function_prefix!("generic_compareEQ"),
            InternalFuctions::GenericCompareNEQ => internal_function_prefix!("generic_compareNEQ"),
            InternalFuctions::GenericCompareLESS => {
                internal_function_prefix!("generic_compareLESS")
            }
            InternalFuctions::GenericCompareLESSEQ => {
                internal_function_prefix!("generic_compareLESSEQ")
            }
            InternalFuctions::GenericCompareGREATER => {
                internal_function_prefix!("generic_compareGREATER")
            }
            InternalFuctions::GenericCompareGREATEREQ => {
                internal_function_prefix!("generic_compareGREATEREQ")
            }
            InternalFuctions::BoolCast => internal_function_prefix!("bool_cast"),
            InternalFuctions::GenericAdd => internal_function_prefix!("generic_add"),
        }
    }
}

pub(super) fn init_internal_functions<'ctx>(
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    //
    // syscalls (MUST be initialized first, used by other internal functions)
    //

    let i32_type = cg.context.i32_type();
    let ptr_type = cg.context.ptr_type(inkwell::AddressSpace::default());

    // puts
    let puts_type = i32_type.fn_type(&[ptr_type.into()], false);
    cg.module
        .add_function(InternalFuctions::Puts.into(), puts_type, None);

    // printf
    let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
    cg.module
        .add_function(InternalFuctions::Printf.into(), printf_type, None);

    // llvm.trap
    let trap_type = cg.context.void_type().fn_type(&[], false);
    cg.module.add_function("llvm.debugtrap", trap_type, None);

    //
    // Internal functions
    //
    register_smollib_funcs_in_module(cg);

    // generic_print
    init_internal_generic_print_function(cg)?;

    // pre init because used in generic function
    let (entry_list_cmp, function_list_cmp) = pre_init_internal_list_cmp_function(cg);
    let function_str_cmp = register_internal_str_cmp_function(cg);

    // generic_compare
    init_internal_compare_generic_function(cg, BinOp::EQ)?;
    init_internal_compare_generic_function(cg, BinOp::NEQ)?;
    init_internal_compare_generic_function(cg, BinOp::LESS)?;
    init_internal_compare_generic_function(cg, BinOp::LESSEQ)?;
    init_internal_compare_generic_function(cg, BinOp::GREATER)?;
    init_internal_compare_generic_function(cg, BinOp::GREATEREQ)?;

    // list_cmp function
    init_internal_list_cmp_function(entry_list_cmp, function_list_cmp, cg)?;
    init_internal_str_cmp_function(function_str_cmp, cg)?;

    // This function is using len should be initialized after the len function
    init_internal_bool_cast_function(cg)?;

    init_internal_add_generic_function(cg)?;

    // smollib user functions
    build_smollib_llvm(cg)?;

    return Ok(());
}

use super::{
    codegen::CodeGen,
    llvm::{init_internal_generic_print_function, LLVMCodegenError},
};

pub enum InternalFuctions {
    Main,
    GenericPrint,
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
pub(super) use internal_function_prefix;

impl Into<&'static str> for InternalFuctions {
    fn into(self) -> &'static str {
        match self {
            InternalFuctions::Main => "main",
            InternalFuctions::GenericPrint => internal_function_prefix!("generic_print"),
            InternalFuctions::Puts => "puts",
            InternalFuctions::Printf => "printf",
            InternalFuctions::Trap => "llvm.debugtrap",
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

    // generic_print
    init_internal_generic_print_function(cg)?;

    return Ok(());
}

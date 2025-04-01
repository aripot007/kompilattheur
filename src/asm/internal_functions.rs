pub enum InternalFuctions {
    Main,
    GenericPrint,
    // Syscalls
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

use super::codegen::CodeGen;

macro_rules! internal_function_prefix {
    ($name: expr) => {
        concat!("__smolpp_f_", $name)
    };
}

impl Into<&'static str> for InternalFuctions {
    fn into(self) -> &'static str {
        match self {
            InternalFuctions::Main => internal_function_prefix!("main"),
            InternalFuctions::GenericPrint => internal_function_prefix!("generic_print"),
            InternalFuctions::Puts => "puts",
            InternalFuctions::Printf => "printf",
        }
    }
}

pub(super) fn init_internal_functions<'ctx>(cg: &CodeGen<'ctx>) {
    //
    // syscalls
    //

    let i32_type = cg.context.i32_type();
    let ptr_type = cg.context.ptr_type(inkwell::AddressSpace::default());
    
    // puts function declaration
    let puts_type = i32_type.fn_type(&[ptr_type.into()], false);
    cg.module
        .add_function(InternalFuctions::Puts.into(), puts_type, None);

    // printf function declaration
    let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
    cg.module
        .add_function(InternalFuctions::Printf.into(), printf_type, None);

}

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
use inkwell::AddressSpace;

use super::codegen::CodeGen;

/// Internal global constants
pub enum InternalGlobalConst {
    
    //
    // Printing strings
    //

    /// String representation for the None type
    NoneString,
    /// String representation for the bool value True
    TrueString,
    /// String representation for the bool value False
    FalseString,

    //
    // Format strings
    //

    /// Format string for printing int
    IntFormatString,
}

macro_rules! internal_global_prefix {
    ($name: expr) => {
        concat!("__smolpp_g_", $name)
    };
}

impl Into<&'static str> for InternalGlobalConst {
    fn into(self) -> &'static str {
        match self {
            InternalGlobalConst::NoneString => internal_global_prefix!("none_string"),
            InternalGlobalConst::TrueString => internal_global_prefix!("true_string"),
            InternalGlobalConst::FalseString => internal_global_prefix!("false_string"),
            InternalGlobalConst::IntFormatString => internal_global_prefix!("int_fmt_string"),
                    }
    }
}

fn create_global_string<'ctx>(name: InternalGlobalConst, value: &str, cg: &CodeGen<'ctx>) {
    let module = cg.context.create_module("my_module");

    let string_value = cg.context.const_string(value.as_bytes(), true);

    // Declare it as a global variable
    let global_var = module.add_global(string_value.get_type(), Some(AddressSpace::default()), name.into());
    global_var.set_initializer(&string_value);
    global_var.set_constant(true);
}

/// Get a global constant registered in the CodeGen.
/// The global consts MUST be initialized before using this macro
macro_rules! get_internal_global_const {
    ($cg: expr, $name: expr) => {
        $cg.module.get_global($name.into()).unwrap()
    };
}
pub(super) use get_internal_global_const;

/// Initialize internal global constants used by smolpp (eg. error strings)
pub(super) fn init_internal_global_consts<'ctx>(cg: &CodeGen<'ctx>) {

    // Printing strings
    create_global_string(InternalGlobalConst::NoneString, "None", cg);
    create_global_string(InternalGlobalConst::TrueString, "True", cg);
    create_global_string(InternalGlobalConst::FalseString, "False", cg);

    // Format strings
    create_global_string(InternalGlobalConst::IntFormatString, "%d", cg);

}

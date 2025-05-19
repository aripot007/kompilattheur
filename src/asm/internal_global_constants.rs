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

    /// Opening brace for list printing
    ListOpeningStr,
    /// Closing brace for list printing
    ListClosingStr,
    /// Closing brace for list printing
    ListSeparatorStr,

    //
    // Format strings
    //
    /// Format string for converting int to string (eg. for concatenation)
    IntFormatString,

    // \n string
    LineReturn,
}

/// Internal global string constants used for runtime error printing
pub enum RuntimeErrorMsg {
    /// Used when an invalid type value is encountered during type comparison.
    ///
    /// Takes the type value as an i8 argument
    PanicInvalidInternalTypeValueFormatString,

    /// Used when we generate something that is not yet implemented
    PanicNotImplemented,

    /// Used when a type error is encountered during generic comparison
    PanicInvalidInternalTypeCompareGeneric,

    /// Used when the type of a variable is not what was expected
    ///
    /// Takes a string message as an argument
    TypeError,

    /// index, length
    IndexOutOfBound,
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
            InternalGlobalConst::ListOpeningStr => {
                internal_global_prefix!("list_open_brace_string")
            }
            InternalGlobalConst::ListClosingStr => {
                internal_global_prefix!("list_close_brace_string")
            }
            InternalGlobalConst::ListSeparatorStr => {
                internal_global_prefix!("list_separator_string")
            }
            InternalGlobalConst::IntFormatString => internal_global_prefix!("int_fmt_string"),
            InternalGlobalConst::LineReturn => internal_global_prefix!("line_return"),
        }
    }
}

impl Into<&'static str> for RuntimeErrorMsg {
    fn into(self) -> &'static str {
        match self {
            RuntimeErrorMsg::PanicInvalidInternalTypeValueFormatString => {
                internal_global_prefix!("panic_invalid_type_fmt_string")
            }
            RuntimeErrorMsg::PanicNotImplemented => internal_global_prefix!("panic_unimplemented"),
            RuntimeErrorMsg::TypeError => internal_global_prefix!("error_type"),
            RuntimeErrorMsg::IndexOutOfBound => internal_global_prefix!("error_out_of_bound"),
            RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric => {
                internal_global_prefix!("panic_invalid_type_compare_generic")
            }
        }
    }
}

pub fn create_global_string<'ctx, T: Into<&'static str>>(name: T, value: &str, cg: &CodeGen<'ctx>) {
    let string_value = cg.context.const_string(value.as_bytes(), true);

    // Declare it as a global variable
    let global_var = cg.module.add_global(
        string_value.get_type(),
        Some(AddressSpace::default()),
        name.into(),
    );
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

    create_global_string(InternalGlobalConst::ListOpeningStr, "[", cg);
    create_global_string(InternalGlobalConst::ListClosingStr, "]", cg);
    create_global_string(InternalGlobalConst::ListSeparatorStr, ", ", cg);

    // Format strings
    create_global_string(InternalGlobalConst::IntFormatString, "%d", cg);
    create_global_string(InternalGlobalConst::LineReturn, "\n", cg);

    // Error messages
    create_global_string(
        RuntimeErrorMsg::PanicInvalidInternalTypeValueFormatString,
        "PANIC: Invalid internal type value %d\n",
        cg,
    );
    create_global_string(
        RuntimeErrorMsg::PanicNotImplemented,
        "PANIC: LLVM not implemented yet\n",
        cg,
    );
    create_global_string(RuntimeErrorMsg::TypeError, "TypeError: %s\n", cg);
    create_global_string(
        RuntimeErrorMsg::IndexOutOfBound,
        "IndexError: index %d out of bounds for list of length %d\n",
        cg,
    );

    create_global_string(
        RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
        "PANIC: Invalid internal type value for generic comparison\n",
        cg,
    );
}

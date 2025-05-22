use colored::Colorize;
use inkwell::AddressSpace;

use super::codegen::CodeGen;

use crate::common::diagnostic::{ERROR_COLOR, HIGHLIGHT_ERROR_COLOR};

/// Internal global constants
#[derive(Clone)]
pub enum InternalGlobalConst {
    /// stdin FILE*
    #[cfg(feature = "smollib-input")]
    StdinFile,

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
    RangeFormatString,

    // \n string
    LineReturn,

    // Types
    IntType,
    StringType,
    BoolType,
    ListType,
    RangeType,
    NoneType,

    // Expected types
    ExpectedTypeNone,
    ExpectedTypeInt,
    ExpectedTypeBool,
    ExpectedTypeString,
    ExpectedTypeList,
    ExpectedTypeRange,

    CanOnlyConcatenate,
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

    /// Used when try to add two different type
    PanicInvalidInternalTypeAddGeneric,

    /// Used when getline fails
    PanicGetlineError,

    /// Used when the type of a variable is not what was expected
    ///
    /// Takes a string message as an argument
    TypeError,
    TypeErrorDyn,

    CompareGreater,
    CompareGreaterEq,
    CompareLess,
    CompareLessEq,

    /// index, length
    IndexOutOfBound,

    //
    InvalidTypeListFunction,

    //
    InvalidStringForIntFunction,

    //
    PanicInvalidInternalTypeInTypeFunction,

    // Used for error messages that gives the line and column of the error
    LocalizeError,
}

macro_rules! internal_global_prefix {
    ($name: expr) => {
        concat!("__smolpp_g_", $name)
    };
}

impl Into<&'static str> for InternalGlobalConst {
    fn into(self) -> &'static str {
        match self {
            #[cfg(feature = "smollib-input")]
            InternalGlobalConst::StdinFile => "stdin",
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
            InternalGlobalConst::RangeFormatString => internal_global_prefix!("range_fmt_string"),
            InternalGlobalConst::LineReturn => internal_global_prefix!("line_return"),
            InternalGlobalConst::IntType => internal_global_prefix!("int_type"),
            InternalGlobalConst::StringType => internal_global_prefix!("string_type"),
            InternalGlobalConst::BoolType => internal_global_prefix!("bool_type"),
            InternalGlobalConst::ListType => internal_global_prefix!("list_type"),
            InternalGlobalConst::RangeType => internal_global_prefix!("range_type"),
            InternalGlobalConst::NoneType => internal_global_prefix!("none_type"),
            InternalGlobalConst::ExpectedTypeNone => internal_global_prefix!("expected_type_none"),
            InternalGlobalConst::ExpectedTypeInt => internal_global_prefix!("expected_type_int"),
            InternalGlobalConst::ExpectedTypeBool => internal_global_prefix!("expected_type_bool"),
            InternalGlobalConst::ExpectedTypeString => {
                internal_global_prefix!("expected_type_string")
            }
            InternalGlobalConst::ExpectedTypeList => internal_global_prefix!("expected_type_list"),
            InternalGlobalConst::ExpectedTypeRange => {
                internal_global_prefix!("expected_type_range")
            }
            InternalGlobalConst::CanOnlyConcatenate => {
                internal_global_prefix!("can_only_concatenate")
            }
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
            RuntimeErrorMsg::PanicGetlineError => internal_global_prefix!("panic_getline_error"),
            RuntimeErrorMsg::TypeError => internal_global_prefix!("error_type"),
            RuntimeErrorMsg::TypeErrorDyn => internal_global_prefix!("error_type_dyn"),
            RuntimeErrorMsg::IndexOutOfBound => internal_global_prefix!("error_out_of_bound"),
            RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric => {
                internal_global_prefix!("panic_invalid_type_compare_generic")
            }
            RuntimeErrorMsg::CompareGreater => internal_global_prefix!("compare_greater"),
            RuntimeErrorMsg::CompareGreaterEq => internal_global_prefix!("compare_greater_eq"),
            RuntimeErrorMsg::CompareLess => internal_global_prefix!("compare_less"),
            RuntimeErrorMsg::CompareLessEq => internal_global_prefix!("compare_less_eq"),
            RuntimeErrorMsg::PanicInvalidInternalTypeAddGeneric => {
                internal_global_prefix!("panic_invalid_type_add_generic")
            }
            RuntimeErrorMsg::InvalidTypeListFunction => {
                internal_global_prefix!("invalid_type_list_function")
            }
            RuntimeErrorMsg::InvalidStringForIntFunction => {
                internal_global_prefix!("invalid_string_value_int_function")
            }
            RuntimeErrorMsg::PanicInvalidInternalTypeInTypeFunction => {
                internal_global_prefix!("panic_invalid_type_in_type_function")
            }
            RuntimeErrorMsg::LocalizeError => internal_global_prefix!("localize_error"),
        }
    }
}

impl InternalGlobalConst {
    pub fn get_value(&self) -> &'static str {
        match self {
            InternalGlobalConst::NoneString => "None",
            InternalGlobalConst::TrueString => "True",
            InternalGlobalConst::FalseString => "False",
            InternalGlobalConst::ListOpeningStr => "[",
            InternalGlobalConst::ListClosingStr => "]",
            InternalGlobalConst::ListSeparatorStr => ", ",
            InternalGlobalConst::IntFormatString => "%d",
            InternalGlobalConst::RangeFormatString => "range(%d)",
            InternalGlobalConst::LineReturn => "\n",
            InternalGlobalConst::IntType => "Int",
            InternalGlobalConst::StringType => "String",
            InternalGlobalConst::BoolType => "Bool",
            InternalGlobalConst::ListType => "List",
            InternalGlobalConst::RangeType => "Range",
            InternalGlobalConst::NoneType => "None",
            InternalGlobalConst::ExpectedTypeNone => "Expected type None",
            InternalGlobalConst::ExpectedTypeInt => "Expected type Int",
            InternalGlobalConst::ExpectedTypeBool => "Expected type Bool",
            InternalGlobalConst::ExpectedTypeString => "Expected type String",
            InternalGlobalConst::ExpectedTypeList => "Expected type List",
            InternalGlobalConst::ExpectedTypeRange => "Expected type Range",
            InternalGlobalConst::CanOnlyConcatenate => "can only concatenate",
            #[cfg(feature = "smollib-input")]
            InternalGlobalConst::StdinFile => panic!("stdin value should not be used"),
        }
    }
}

fn create_global_string<'ctx>(name: InternalGlobalConst, cg: &CodeGen<'ctx>) {
    let string_value = cg.context.const_string(name.get_value().as_bytes(), true);

    // Declare it as a global variable
    let global_var = cg.module.add_global(
        string_value.get_type(),
        Some(AddressSpace::default()),
        name.into(),
    );
    global_var.set_initializer(&string_value);
    global_var.set_constant(true);
}

fn create_global_error_string<'ctx>(name: RuntimeErrorMsg, value: &str, cg: &CodeGen<'ctx>) {
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
pub(crate) use get_internal_global_const;

/// Initialize internal global constants used by smolpp (eg. error strings)
pub(super) fn init_internal_global_consts<'ctx>(cg: &CodeGen<'ctx>) {
    #[cfg(feature = "smollib-input")]
    {
        // stdin FILE*
        let file_ptr_type = cg.context.ptr_type(AddressSpace::default()); // Treat FILE* as i8*

        let stdin_global =
            cg.module
                .add_global(file_ptr_type, None, InternalGlobalConst::StdinFile.into());
        stdin_global.set_linkage(inkwell::module::Linkage::External);
    }

    // Printing strings
    create_global_string(InternalGlobalConst::NoneString, cg);
    create_global_string(InternalGlobalConst::TrueString, cg);
    create_global_string(InternalGlobalConst::FalseString, cg);

    create_global_string(InternalGlobalConst::ListOpeningStr, cg);
    create_global_string(InternalGlobalConst::ListClosingStr, cg);
    create_global_string(InternalGlobalConst::ListSeparatorStr, cg);

    // Format strings
    create_global_string(InternalGlobalConst::IntFormatString, cg);
    create_global_string(InternalGlobalConst::RangeFormatString, cg);
    create_global_string(InternalGlobalConst::LineReturn, cg);

    // Types
    create_global_string(InternalGlobalConst::IntType, cg);
    create_global_string(InternalGlobalConst::StringType, cg);
    create_global_string(InternalGlobalConst::BoolType, cg);
    create_global_string(InternalGlobalConst::ListType, cg);
    create_global_string(InternalGlobalConst::RangeType, cg);
    create_global_string(InternalGlobalConst::NoneType, cg);

    // Expected types
    create_global_string(InternalGlobalConst::ExpectedTypeNone, cg);
    create_global_string(InternalGlobalConst::ExpectedTypeInt, cg);
    create_global_string(InternalGlobalConst::ExpectedTypeBool, cg);
    create_global_string(InternalGlobalConst::ExpectedTypeString, cg);
    create_global_string(InternalGlobalConst::ExpectedTypeList, cg);
    create_global_string(InternalGlobalConst::ExpectedTypeRange, cg);

    create_global_string(InternalGlobalConst::CanOnlyConcatenate, cg);

    // Error messages
    create_global_error_string(
        RuntimeErrorMsg::PanicInvalidInternalTypeValueFormatString,
        format!(
            "{} {}{}",
            "PANIC:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Invalid internal type value %d".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::PanicNotImplemented,
        format!(
            "{} {}{}",
            "PANIC:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Not implemented yet".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::PanicGetlineError,
        format!(
            "{} {}{}",
            "PANIC:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Could not read from standard input".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::PanicGetlineError,
        format!(
            "{} {}{}",
            "PANIC:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Could not read from standard input".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::TypeError,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "%s but got %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::TypeErrorDyn,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "%s %s (not %s) to %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::CompareGreater,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "'>' not supported between instances of %s and %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::CompareGreaterEq,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "'>=' not supported between instances of %s and %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::CompareLess,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "'<' not supported between instances of %s and %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::CompareLessEq,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "'<=' not supported between instances of %s and %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::IndexOutOfBound,
        format!(
            "{} {}{}",
            "IndexError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "index %d out of bounds for list of length %d".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
        format!(
            "{} {}{}",
            "PANIC:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Invalid internal type value for generic comparison".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::PanicInvalidInternalTypeAddGeneric,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "unsupported operand type(s) for +: %s and %s".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::InvalidTypeListFunction,
        format!(
            "{} {}{}",
            "TypeError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "%s object is not iterable".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::InvalidStringForIntFunction,
        format!(
            "{} {}{}",
            "InvalidIntError:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Invalid string for int function".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::PanicInvalidInternalTypeInTypeFunction,
        format!(
            "{} {}{}",
            "PANIC:"
                .truecolor(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2)
                .bold(),
            "Invalid internal type value for type function (bitmask: %s)".truecolor(
                HIGHLIGHT_ERROR_COLOR.0,
                HIGHLIGHT_ERROR_COLOR.1,
                HIGHLIGHT_ERROR_COLOR.2
            ),
            "\x1b[0m\n"
        )
        .as_str(),
        cg,
    );

    create_global_error_string(
        RuntimeErrorMsg::LocalizeError,
        format!("{} {}{}", "At line :".bold(), " %d:%d", "\x1b[0m\n").as_str(),
        cg,
    );
}

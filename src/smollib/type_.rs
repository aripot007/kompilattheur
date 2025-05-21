use inkwell::{types::FunctionType, values::FunctionValue};

use crate::asm::llvm::panic::smolpp_panic_with_unreachable;
use crate::asm::RuntimeErrorMsg;
use crate::common::localizable::LocalizationInfo;
use crate::{
    asm::{codegen::CodeGen, get_internal_global_const, InternalGlobalConst, LLVMCodegenError},
    typing::{Function, Type, Weak},
};

use super::SmollibFunction;

pub(super) struct SmolType {}

impl SmollibFunction for SmolType {
    fn name(&self) -> &str {
        "type"
    }

    fn func_type(&self) -> Function {
        let arg_type = Weak::new_with_possible(&[
            Type::None,
            Type::Bool,
            Type::Int,
            Type::String,
            Type::List,
            Type::Range,
        ])
        .locked();
        Function {
            args: vec![Type::Weak(arg_type)],
            returns: Type::String,
        }
    }

    fn llvm_type<'ctx>(&self, cg: &CodeGen<'ctx>) -> FunctionType<'ctx> {
        let var_type = cg.smolpp_types.dynamic_type;
        let func_type = var_type.fn_type(&vec![var_type.into(); 1], false);
        return func_type;
    }

    fn build_llvm<'ctx>(
        &self,
        function: FunctionValue<'ctx>,
        cg: &mut CodeGen<'ctx>,
    ) -> Result<(), LLVMCodegenError> {
        let entry = cg.context.append_basic_block(function, "type_entry");
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
        create_string_return(cg, InternalGlobalConst::NoneType)?;

        cg.builder.position_at_end(case_bool);
        create_string_return(cg, InternalGlobalConst::BoolType)?;

        cg.builder.position_at_end(case_int);
        create_string_return(cg, InternalGlobalConst::IntType)?;

        cg.builder.position_at_end(case_string);
        create_string_return(cg, InternalGlobalConst::StringType)?;

        cg.builder.position_at_end(case_list);
        create_string_return(cg, InternalGlobalConst::ListType)?;

        cg.builder.position_at_end(case_range);
        create_string_return(cg, InternalGlobalConst::RangeType)?;

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
}

pub fn create_string_return<'ctx>(
    cg: &CodeGen<'ctx>,
    type_field: InternalGlobalConst,
) -> Result<(), LLVMCodegenError> {
    let fmt_string = get_internal_global_const!(cg, type_field.clone()).as_pointer_value();
    let string_len = cg
        .context
        .i64_type()
        .const_int(type_field.get_value().len() as u64, false);
    let smol_string = cg.build_string_struct(string_len, fmt_string)?;
    let smol_string_ptr = cg
        .builder
        .build_malloc(cg.smolpp_types.string_type, "string_struct_ptr")?;
    cg.builder.build_store(smol_string_ptr, smol_string)?;
    let smol_string_ptr =
        cg.builder
            .build_ptr_to_int(smol_string_ptr, cg.context.i64_type(), "smol_string_ptr")?;
    let smol_var = cg.create_variable(Type::String, smol_string_ptr)?;
    cg.builder.build_return(Some(&smol_var))?;
    return Ok(());
}

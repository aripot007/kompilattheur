use inkwell::{types::FunctionType, values::FunctionValue, AddressSpace, IntPredicate};

use crate::{
    asm::{
        codegen::CodeGen, llvm::panic::smolpp_panic_with_unreachable, InternalFuctions,
        InternalGlobalConst, LLVMCodegenError, RuntimeErrorMsg,
    },
    common::localizable::LocalizationInfo,
    typing::{Function, Type},
};

use super::SmollibFunction;

pub(super) struct SmolInput {}

impl SmollibFunction for SmolInput {
    fn name(&self) -> &str {
        "input"
    }

    fn func_type(&self) -> Function {
        Function {
            args: Vec::new(),
            returns: Type::String,
        }
    }

    fn llvm_type<'ctx>(&self, cg: &CodeGen<'ctx>) -> FunctionType<'ctx> {
        let var_type = cg.smolpp_types.dynamic_type;
        let func_type = var_type.fn_type(&[], false);
        return func_type;
    }

    fn build_llvm<'ctx>(
        &self,
        function: FunctionValue<'ctx>,
        cg: &mut CodeGen<'ctx>,
    ) -> Result<(), LLVMCodegenError> {
        let entry = cg.context.append_basic_block(function, "input_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let ptr_type = cg.context.ptr_type(AddressSpace::default());
        let i64_type = cg.context.i64_type();
        let i8_type = cg.context.i8_type();

        let stdin_global = cg
            .module
            .get_global(InternalGlobalConst::StdinFile.into())
            .unwrap();
        let stdin_ptr = cg
            .builder
            .build_load(ptr_type, stdin_global.as_pointer_value(), "stdin")?
            .into_pointer_value();

        let getline_func = cg
            .module
            .get_function(InternalFuctions::Getline.into())
            .unwrap();

        // char**
        let line_ptr = cg.builder.build_alloca(ptr_type, "line_ptr")?;
        cg.builder.build_store(line_ptr, ptr_type.const_zero())?;

        // int*
        let capa_ptr = cg.builder.build_alloca(i64_type, "capa_ptr")?;
        cg.builder.build_store(capa_ptr, i64_type.const_zero())?;

        let call = cg.builder.build_call(
            getline_func,
            &[line_ptr.into(), capa_ptr.into(), stdin_ptr.into()],
            "getline_call",
        )?;

        // Handle getline errors
        let getline_error_block = cg.context.append_basic_block(function, "getline_error");
        let getline_success_block = cg.context.append_basic_block(function, "getline_success");

        let len = call.try_as_basic_value().unwrap_left().into_int_value();

        let cdt = cg.builder.build_int_compare(
            IntPredicate::SLT,
            len,
            i64_type.const_zero(),
            "getline_failed",
        )?;

        cg.builder
            .build_conditional_branch(cdt, getline_error_block, getline_success_block)?;

        cg.builder.position_at_end(getline_error_block);
        smolpp_panic_with_unreachable::<LocalizationInfo>(
            cg,
            RuntimeErrorMsg::PanicGetlineError,
            &[],
            None,
        )?;

        cg.builder.position_at_end(getline_success_block);

        // char*
        let loaded_line = cg
            .builder
            .build_load(ptr_type, line_ptr, "loaded_line")?
            .into_pointer_value();
        // let loaded_capa = cg.builder.build_load(i64_type, capa_ptr, "loaded_capa")?.into_int_value();

        // Replace last '\n' with '\0'
        let last_char_ptr = unsafe {
            cg.builder
                .build_in_bounds_gep(i8_type, loaded_line, &[len], "last_char_ptr")
        }?;
        cg.builder
            .build_store(last_char_ptr, i8_type.const_zero())?;

        // Update len
        let len = cg
            .builder
            .build_int_sub(len, i64_type.const_int(1, false), "len")?;

        let str_struct = cg.build_string_struct(len, loaded_line)?;
        let str_struct_ptr = cg
            .builder
            .build_malloc(cg.smolpp_types.string_type, "str_struct_ptr")?;
        cg.builder.build_store(str_struct_ptr, str_struct)?;

        // Build variable
        let str_struct_ptr_int =
            cg.builder
                .build_ptr_to_int(str_struct_ptr, i64_type, "str_struct_ptr_int")?;

        let res_var = cg.create_variable(Type::String, str_struct_ptr_int)?;

        cg.builder.build_return(Some(&res_var))?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

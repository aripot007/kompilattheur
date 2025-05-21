use inkwell::{types::FunctionType, values::FunctionValue};

use crate::{
    asm::{codegen::CodeGen, llvm::assert_type::assert_type, LLVMCodegenError},
    common::localizable::LocalizationInfo,
    typing::{Function, Type},
};

use super::SmollibFunction;

pub(super) struct SmolRange {}

impl SmollibFunction for SmolRange {
    fn name(&self) -> &str {
        "range"
    }

    fn func_type(&self) -> Function {
        Function {
            args: vec![Type::Int],
            returns: Type::Range,
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
        let entry = cg.context.append_basic_block(function, "range_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let var1 = function
            .get_nth_param(0 as u32)
            .unwrap()
            .into_struct_value();

        assert_type::<LocalizationInfo>(Type::Int, &var1, cg, None, None)?;

        let var1_value = cg.get_variable_value(var1)?.into_int_value();

        let return_value = cg.create_variable(Type::Range, var1_value)?;
        cg.builder.build_return(Some(&return_value))?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

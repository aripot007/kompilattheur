use inkwell::{types::FunctionType, values::FunctionValue};

use crate::{
    asm::{codegen::CodeGen, llvm::assert_type_oneof, LLVMCodegenError},
    typing::{Function, Type, Weak},
};

use super::SmollibFunction;

pub(super) struct SmolInt {}

impl SmollibFunction for SmolInt {
    fn name(&self) -> &str {
        "int"
    }

    fn func_type(&self) -> Function {
        let arg_type = Weak::new_with_possible(&[Type::String, Type::Int]).locked();
        Function {
            args: vec![Type::Weak(arg_type)],
            returns: Type::Int,
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
        let entry = cg.context.append_basic_block(function, "int_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let var1 = function
            .get_nth_param(0 as u32)
            .unwrap()
            .into_struct_value();

        assert_type_oneof(&[Type::String, Type::Int], &var1, cg, None)?;

        // ---

        // def int(n):
        //   if typeof(n) == int:
        //     return n
        //   if typeof(n) == string:
        //     res = 0
        //     for i in n:
        //       if not("0" <= i <= "9"):
        //         return error
        //       res = res * 10 + i
        //     return res

        // ---

        let return_value = cg.create_variable(Type::Int, interator_value_len)?;
        cg.builder.build_return(Some(&return_value))?;

        // Error handling
        cg.builder.position_at_end(error_block);
        // Create an error value or use a default value like 0
        let error_value = cg.create_variable(Type::Int, cg.context.i64_type().const_zero())?;
        cg.builder.build_return(Some(&error_value))?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

use inkwell::{types::FunctionType, values::FunctionValue, AddressSpace};

use crate::{
    asm::{codegen::CodeGen, llvm::assert_type::assert_type_oneof, LLVMCodegenError},
    common::localizable::LocalizationInfo,
    typing::{Function, Type, Weak},
};

use super::SmollibFunction;

pub(super) struct SmolLen {}

impl SmollibFunction for SmolLen {
    fn name(&self) -> &str {
        "len"
    }

    fn func_type(&self) -> Function {
        let arg_type = Weak::new_with_possible(&[Type::List, Type::String]).locked();
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
        let entry = cg.context.append_basic_block(function, "len_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let var1 = function
            .get_nth_param(0 as u32)
            .unwrap()
            .into_struct_value();

        assert_type_oneof::<LocalizationInfo>(&[Type::List, Type::String], &var1, cg, None, None)?;

        let var1_value = cg.get_variable_value(var1)?.into_int_value();
        // Get the length of the list or string
        let ptr_type = cg.context.ptr_type(AddressSpace::default());

        let var1_ptr = cg
            .builder
            .build_int_to_ptr(var1_value, ptr_type, "list_ptr")?;

        // Load the SmolList
        let var1_list = cg
            .builder
            .build_load(cg.smolpp_types.list_type, var1_ptr, "list")?
            .into_struct_value();

        let interator_value_len = cg.build_get_list_length(var1_list)?;

        let return_value = cg.create_variable(Type::Int, interator_value_len)?;
        cg.builder.build_return(Some(&return_value))?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

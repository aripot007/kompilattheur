use inkwell::{types::FunctionType, values::FunctionValue, AddressSpace};

use crate::{
    asm::{codegen::CodeGen, llvm::assert_type::assert_type_oneof, LLVMCodegenError},
    common::localizable::LocalizationInfo,
    typing::{Function, Type, Weak},
};

use super::SmollibFunction;

pub(super) struct SmolIsInt {}

impl SmollibFunction for SmolIsInt {
    fn name(&self) -> &str {
        "is_int"
    }

    fn func_type(&self) -> Function {
        let arg_type = Weak::new_with_possible(&[Type::String, Type::Int, Type::Bool]).locked();
        Function {
            args: vec![Type::Weak(arg_type)],
            returns: Type::Bool,
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
        let entry = cg.context.append_basic_block(function, "is_int_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let var1 = function
            .get_nth_param(0 as u32)
            .unwrap()
            .into_struct_value();

        assert_type_oneof::<LocalizationInfo>(&[Type::String], &var1, cg, None, None)?;

        // ---

        // def int(n):
        //   if typeof(n) == bool:
        //     if n:
        //       return 1
        //     return 0
        //   if typeof(n) == int:
        //     return n
        //   if typeof(n) == string:
        //     res = 0
        //     for i in n:
        //       if not("0" <= i <= "9"):
        //         return error
        //       res = res * 10 + i
        //     return res

        // Create basic blocks for each type
        let int_block = cg.context.append_basic_block(function, "int_block");
        let bool_block = cg.context.append_basic_block(function, "bool_block");
        let string_block = cg.context.append_basic_block(function, "string_block");
        let error_block = cg.context.append_basic_block(function, "error_block");

        // Get the type of the input value
        let type_field = cg.get_variable_type(var1)?;

        // Check if type is INT
        let is_int = cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_field,
            cg.context
                .i8_type()
                .const_int(Type::Int.get_bitmask() as u64, false),
            "is_int",
        )?;

        let not_int_block = cg.context.append_basic_block(function, "not_int_block");

        cg.builder
            .build_conditional_branch(is_int, int_block, not_int_block)?;

        cg.builder.position_at_end(int_block);
        let return_int_value =
            cg.create_variable(Type::Bool, cg.context.i64_type().const_int(1, false))?;
        cg.builder.build_return(Some(&return_int_value))?;

        // Check if type is BOOL
        cg.builder.position_at_end(not_int_block);

        let is_bool = cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_field,
            cg.context
                .i8_type()
                .const_int(Type::Bool.get_bitmask() as u64, false),
            "is_bool",
        )?;

        let not_bool_block = cg.context.append_basic_block(function, "not_bool_block");

        cg.builder
            .build_conditional_branch(is_bool, bool_block, not_bool_block)?;

        cg.builder.position_at_end(bool_block);
        let return_bool_value =
            cg.create_variable(Type::Bool, cg.context.i64_type().const_int(1, false))?;
        cg.builder.build_return(Some(&return_bool_value))?;

        // Check if type is STRING
        cg.builder.position_at_end(not_bool_block);
        let is_string = cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_field,
            cg.context
                .i8_type()
                .const_int(Type::String.get_bitmask() as u64, false),
            "is_string",
        )?;

        cg.builder
            .build_conditional_branch(is_string, string_block, error_block)?;

        // String conversion code
        cg.builder.position_at_end(string_block);

        // Extract the string from the var
        let string_ptr = cg.get_variable_value(var1)?.into_int_value();
        let string_ptr = cg.builder.build_int_to_ptr(
            string_ptr,
            cg.context.ptr_type(AddressSpace::default()),
            "string_ptr",
        )?;
        let string_struct = cg
            .builder
            .build_load(cg.smolpp_types.string_type, string_ptr, "string_struct")?
            .into_struct_value();

        // Get the string length and array pointer
        let string_len = cg.build_get_string_length(string_struct)?;
        let string_array_ptr = cg.build_get_string_array_ptr(string_struct)?;

        // Create the loop blocks
        let loop_header = cg.context.append_basic_block(function, "loop_header");
        let loop_body = cg.context.append_basic_block(function, "loop_body");
        let loop_digit_check = cg.context.append_basic_block(function, "loop_digit_check");
        let loop_inc = cg.context.append_basic_block(function, "loop_inc");
        let loop_exit = cg.context.append_basic_block(function, "loop_exit");

        // Create the loop index
        let index_ptr = cg
            .builder
            .build_alloca(cg.context.i64_type(), "index_ptr")?;
        cg.builder
            .build_store(index_ptr, cg.context.i64_type().const_zero())?;

        // Jump to the loop header
        cg.builder.build_unconditional_branch(loop_header)?;

        // Loop header: check if index < length
        cg.builder.position_at_end(loop_header);
        let current_index =
            cg.builder
                .build_load(cg.context.i64_type(), index_ptr, "current_index")?;
        let loop_condition = cg.builder.build_int_compare(
            inkwell::IntPredicate::ULT,
            current_index.into_int_value(),
            string_len,
            "loop_condition",
        )?;
        cg.builder
            .build_conditional_branch(loop_condition, loop_body, loop_exit)?;

        // Loop body: get the current character
        cg.builder.position_at_end(loop_body);
        let char_ptr = unsafe {
            cg.builder.build_gep(
                cg.context.i8_type(),
                string_array_ptr,
                &[current_index.into_int_value()],
                "char_ptr",
            )
        }?;
        let current_char = cg
            .builder
            .build_load(cg.context.i8_type(), char_ptr, "current_char")?;

        // Check if character is between '0' and '9'
        let char_gte_zero = cg.builder.build_int_compare(
            inkwell::IntPredicate::UGE,
            current_char.into_int_value(),
            cg.context.i8_type().const_int('0' as u64, false),
            "char_gte_zero",
        )?;
        let char_lte_nine = cg.builder.build_int_compare(
            inkwell::IntPredicate::ULE,
            current_char.into_int_value(),
            cg.context.i8_type().const_int('9' as u64, false),
            "char_lte_nine",
        )?;
        let is_digit = cg
            .builder
            .build_and(char_gte_zero, char_lte_nine, "is_digit")?;
        cg.builder
            .build_conditional_branch(is_digit, loop_digit_check, error_block)?;

        // Process the digit
        cg.builder.position_at_end(loop_digit_check);

        // Increment the index
        cg.builder.build_unconditional_branch(loop_inc)?;

        // Loop increment
        cg.builder.position_at_end(loop_inc);
        let next_index = cg.builder.build_int_add(
            current_index.into_int_value(),
            cg.context.i64_type().const_int(1, false),
            "next_index",
        )?;
        cg.builder.build_store(index_ptr, next_index)?;
        cg.builder.build_unconditional_branch(loop_header)?;

        // Loop exit: create and return the result
        cg.builder.position_at_end(loop_exit);

        let return_value =
            cg.create_variable(Type::Bool, cg.context.i64_type().const_int(1, false))?;
        cg.builder.build_return(Some(&return_value))?;

        // Error handling
        cg.builder.position_at_end(error_block);

        let return_value = cg.create_variable(Type::Bool, cg.context.i64_type().const_zero())?;
        cg.builder.build_return(Some(&return_value))?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

use inkwell::{types::FunctionType, values::FunctionValue, AddressSpace};

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{assert_type::assert_type_oneof, panic::smolpp_panic_with_unreachable},
        LLVMCodegenError, RuntimeErrorMsg,
    },
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

        // Check if input is already an int
        let int_block = cg.context.append_basic_block(function, "int_block");
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

        // Branch based on type
        cg.builder
            .build_conditional_branch(is_int, int_block, string_block)?;

        // If int, just return the value
        cg.builder.position_at_end(int_block);
        let int_value = cg.get_variable_value(var1)?;
        let return_int_value = cg.create_variable(Type::Int, int_value.into_int_value())?;
        cg.builder.build_return(Some(&return_int_value))?;

        // If string, convert to int
        cg.builder.position_at_end(string_block);

        // Check if type is STRING
        let is_string = cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_field,
            cg.context
                .i8_type()
                .const_int(Type::String.get_bitmask() as u64, false),
            "is_string",
        )?;

        // Branch to string conversion or error
        let string_convert_block = cg.context.append_basic_block(function, "string_convert");

        // Branch to error if not a string
        cg.builder
            .build_conditional_branch(is_string, string_convert_block, error_block)?;

        // String conversion code
        cg.builder.position_at_end(string_convert_block);

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

        // Initialize the result to 0
        let result_ptr = cg
            .builder
            .build_alloca(cg.context.i64_type(), "result_ptr")?;
        cg.builder
            .build_store(result_ptr, cg.context.i64_type().const_zero())?;

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

        // Convert char to digit value (subtract '0')
        let digit_value = cg.builder.build_int_sub(
            current_char.into_int_value(),
            cg.context.i8_type().const_int('0' as u64, false),
            "digit_value",
        )?;
        let digit_value_ext =
            cg.builder
                .build_int_z_extend(digit_value, cg.context.i64_type(), "digit_value_ext")?;

        // result = result * 10 + digit_value
        let current_result =
            cg.builder
                .build_load(cg.context.i64_type(), result_ptr, "current_result")?;
        let result_times_10 = cg.builder.build_int_mul(
            current_result.into_int_value(),
            cg.context.i64_type().const_int(10, false),
            "result_times_10",
        )?;
        let new_result =
            cg.builder
                .build_int_add(result_times_10, digit_value_ext, "new_result")?;
        cg.builder.build_store(result_ptr, new_result)?;

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
        let final_result =
            cg.builder
                .build_load(cg.context.i64_type(), result_ptr, "final_result")?;
        let return_value = cg.create_variable(Type::Int, final_result.into_int_value())?;
        cg.builder.build_return(Some(&return_value))?;

        // Error handling
        cg.builder.position_at_end(error_block);

        smolpp_panic_with_unreachable(
            cg,
            RuntimeErrorMsg::InvalidStringForIntFunction,
            &[type_field.into()],
        )?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

use inkwell::{
    types::FunctionType,
    values::{BasicValue, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{
            assert_type::assert_type_oneof, panic::smolpp_panic_with_unreachable, smolvar::SmolVar,
        },
        LLVMCodegenError, RuntimeErrorMsg,
    },
    typing::{Function, Type, Weak},
};

use super::SmollibFunction;

pub(super) struct SmolList {}

impl SmollibFunction for SmolList {
    fn name(&self) -> &str {
        "list"
    }

    fn func_type(&self) -> Function {
        let arg_type = Weak::new_with_possible(&[Type::List, Type::String, Type::Range]).locked();
        Function {
            args: vec![Type::Weak(arg_type)],
            returns: Type::List,
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
        let entry = cg.context.append_basic_block(function, "list_entry");
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        let var1 = function
            .get_nth_param(0 as u32)
            .unwrap()
            .into_struct_value();

        assert_type_oneof(&[Type::List, Type::String, Type::Range], &var1, cg, None)?;

        // Si type == Range => return Create the list with the range

        let t1 = cg.get_variable_type(var1)?;

        let case_range = cg
            .context
            .append_basic_block(function, "list_function_case_int");
        let case_string_or_list = cg
            .context
            .append_basic_block(function, "list_function_case_string_or_list");
        let default_block = cg
            .context
            .append_basic_block(function, "list_function_default");

        let i8_type = cg.context.i8_type();

        cg.builder.build_switch(
            t1,
            default_block,
            &[
                (
                    i8_type.const_int(Type::None.get_bitmask().into(), false),
                    default_block,
                ),
                (
                    i8_type.const_int(Type::Bool.get_bitmask().into(), false),
                    default_block,
                ),
                (
                    i8_type.const_int(Type::Int.get_bitmask().into(), false),
                    default_block,
                ),
                (
                    i8_type.const_int(Type::String.get_bitmask().into(), false),
                    case_string_or_list,
                ),
                (
                    i8_type.const_int(Type::List.get_bitmask().into(), false),
                    case_string_or_list,
                ),
                (
                    i8_type.const_int(Type::Range.get_bitmask().into(), false),
                    case_range,
                ),
            ],
        )?;

        cg.builder.position_at_end(case_range);
        let result = create_list_variable_from_range(cg, var1)?;
        cg.builder.build_return(Some(&result))?;

        cg.builder.position_at_end(case_string_or_list);
        // Si String ou List => return the same value
        cg.builder.build_return(Some(&var1))?;

        // Default case, print error message
        cg.builder.position_at_end(default_block);
        smolpp_panic_with_unreachable(cg, RuntimeErrorMsg::InvalidTypeListFunction, &[t1.into()])?;

        // Return builder to main block because it's init function
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
        return Ok(());
    }
}

fn create_list_variable_from_range<'ctx>(
    cg: &mut CodeGen<'ctx>,
    var1: SmolVar<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // TODO: Assert if var1 < i32 max
    let capa = cg.get_variable_value(var1)?.into_int_value();
    let (val, list_struct_ptr) = cg.build_list_variable(capa, true)?;

    // Update len
    let len_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        0,
        "create_list_from_range_len_ptr",
    )?;
    cg.builder.build_store(len_ptr, capa)?;

    let array_ptr_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        2,
        "array_ptr_ptr",
    )?;

    loop_to_create_list_from_range(cg, capa, array_ptr_ptr)?;

    return Ok(val);
}

fn loop_to_create_list_from_range<'ctx>(
    cg: &mut CodeGen<'ctx>,
    capa: IntValue<'ctx>,
    array_ptr_ptr: PointerValue<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Get parent block
    let parent_block = cg.builder.get_insert_block().unwrap();
    // Create the loop
    let for_loop_block = cg
        .context
        .insert_basic_block_after(parent_block, "for_loop_block");
    let for_exit = cg
        .context
        .insert_basic_block_after(for_loop_block, "for_loop_exit");

    // Create the internal index variable
    let internal_index_int = cg
        .builder
        .build_alloca(cg.context.i64_type(), "internal_index")?;
    cg.builder
        .build_store(internal_index_int, cg.context.i64_type().const_zero())?;

    // Compare if the len of the iterator is equal to 0
    let guard_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        cg.context.i64_type().const_zero(),
        capa,
        "guard_comparison",
    )?;

    cg.builder
        .build_conditional_branch(guard_comparison, for_exit, for_loop_block)?;

    cg.builder.position_at_end(for_loop_block);

    // Load the intenal index value
    let internal_index_int_load = cg
        .builder
        .build_load(
            cg.context.i64_type(),
            internal_index_int,
            "internal_index_load",
        )?
        .into_int_value();

    // Cast the internal index to i32
    let list_index =
        cg.builder
            .build_int_cast(internal_index_int_load, cg.context.i32_type(), "list_index")?;

    let array_ptr = cg.builder.build_load(
        cg.context.ptr_type(AddressSpace::default()),
        array_ptr_ptr,
        "array_ptr",
    )?;

    // Load the intenal index value
    let elt_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            array_ptr.into_pointer_value(),
            &[list_index],
            "list_from_range_elt_ptr",
        )
    }?;

    let list_value =
        cg.create_variable(Type::Int, internal_index_int_load.as_basic_value_enum())?;

    cg.builder.build_store(elt_ptr, list_value)?;

    // Increment the internal index variable
    let increment_one = cg.builder.build_int_add(
        internal_index_int_load,
        cg.context.i64_type().const_int(1, false),
        "increment_one",
    )?;

    cg.builder.build_store(internal_index_int, increment_one)?;

    // Compare the internal index with the iterator length
    let loop_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        increment_one,
        capa,
        "loop_comparison",
    )?;

    cg.builder
        .build_conditional_branch(loop_comparison, for_loop_block, for_exit)?;

    cg.builder.position_at_end(for_exit);

    return Ok(());
}

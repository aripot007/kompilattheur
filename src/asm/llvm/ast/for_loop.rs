use inkwell::AddressSpace;

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{assert_type, smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::For,
    common::symbol_table::{get_symbol, Symbol},
    typing::Type,
};

use super::{llvm_compute_expr, llvm_from_block};

pub fn llvm_from_for_loop<'ctx>(
    for_loop: &For,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Allocate memory
    let var_ptr = cg.builder.build_alloca(
        cg.smolpp_types.dynamic_type,
        format!("alloca_loop_var_{}", for_loop.var.element.name).as_str(),
    )?;
    // Store initial value with correct type
    let val = cg.create_variable(Type::Any, cg.context.i64_type().const_zero())?;
    cg.builder.build_store(var_ptr, val)?;

    // Register the pointer in the codegen context and update its reference in the symbol table
    let var_ptr_id = Some(cg.register_pointer(var_ptr));

    // Update the symbol table with the pointer
    let block = &for_loop.block;
    let new_symbol = match &block.symbol_table {
        Some(table_tree) => {
            let mut symbol_option = get_symbol(table_tree, &for_loop.var.element.id);
            let symbol_table_elem = match symbol_option {
                Some(ref mut symbol) => symbol,
                None => panic!("Symbol not found in block"),
            };
            symbol_table_elem.symbol = match symbol_table_elem.symbol {
                Symbol::Variable { offset, .. } => Symbol::Variable {
                    offset,
                    ptr_id: var_ptr_id,
                },
                _ => panic!("Expected variable symbol"),
            };
            symbol_table_elem.clone()
        }
        None => panic!("Symbol table not initialized in block"),
    };

    block
        .symbol_table
        .as_ref()
        .unwrap()
        .borrow_mut()
        .insert_symbol(for_loop.var.element.id, new_symbol);

    // Get the iterator value
    let iterator_variable: SmolVar<'ctx> = llvm_compute_expr(&for_loop.iterator, cg)?;

    // Check if the iterator_value is a list
    assert_type(Type::List, &iterator_variable, cg, None)?;

    let iterator_value = cg.get_variable_value(iterator_variable)?;

    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let iterator_ptr =
        cg.builder
            .build_int_to_ptr(iterator_value.into_int_value(), ptr_type, "list_ptr")?;

    // Load the SmolList
    let iterator_list = cg
        .builder
        .build_load(cg.smolpp_types.list_type, iterator_ptr, "list")?
        .into_struct_value();

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

    let interator_value_len = cg.build_get_list_length(iterator_list)?;

    // Compare if the len of the iterator is equal to 0
    let guard_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        cg.context.i64_type().const_zero(),
        interator_value_len,
        "guard_comparison",
    )?;

    cg.builder
        .build_conditional_branch(guard_comparison, for_exit, for_loop_block)?;

    cg.builder.position_at_end(for_loop_block);

    // Get the the current value of the iterator list

    let iterator_loop_ptr = cg.build_get_list_array_ptr(iterator_list)?;

    // Load the intenal index value
    let internal_index_int_load = cg
        .builder
        .build_load(
            cg.context.i64_type(),
            internal_index_int,
            "internal_index_load",
        )?
        .into_int_value();

    // Get iterator[i]
    let iterator_i_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            iterator_loop_ptr,
            &[internal_index_int_load],
            "list_i_ptr",
        )
    }?;

    let iterator_i =
        cg.builder
            .build_load(cg.smolpp_types.dynamic_type, iterator_i_ptr, "list_i")?;

    cg.builder.build_store(var_ptr, iterator_i)?;

    llvm_from_block(&for_loop.block, cg)?;

    // Increment the internal index variable
    let increment_one = cg.builder.build_int_add(
        internal_index_int_load,
        cg.context.i64_type().const_int(1, false),
        "increment_one",
    )?;

    cg.builder.build_store(internal_index_int, increment_one)?;

    let interator_value_len_loop = cg.build_get_list_length(iterator_list)?;

    // Compare the internal index with the iterator length
    let loop_comparison = cg.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        increment_one,
        interator_value_len_loop,
        "loop_comparison",
    )?;

    cg.builder
        .build_conditional_branch(loop_comparison, for_loop_block, for_exit)?;

    cg.builder.position_at_end(for_exit);

    return Ok(());
}

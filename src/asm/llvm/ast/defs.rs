use crate::asm::{codegen::CodeGen, llvm::LLVMCodegenError};
use crate::ast::nodes::Defs;
use crate::common::symbol_table::Symbol;

use super::llvm_from_block;

pub fn llvm_from_defs<'ctx>(defs: &Defs, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    // TODO: init signature then code
    for def in &defs.defs {
        let name = def.identifier.element.name.clone();
        let var_type = cg.smolpp_types.dynamic_type;

        // Register the function in the module
        let func_type = var_type.fn_type(&vec![var_type.into(); def.params.len()], false);
        let function = cg.module.add_function(
            format!("__smolpp_user_f_{}", name).as_str(),
            func_type,
            None,
        );

        // Build the function
        let entry = cg.context.append_basic_block(function, "function_entry");

        // Switch builder to the function block
        cg.builder.position_at_end(entry);
        cg.current_function = function;

        // Process function parameters
        if let Some(table_tree) = &def.block.symbol_table {
            let mut symbol_table = table_tree.borrow().get_value();

            // Iterate through function parameters
            for (i, param) in def.params.iter().enumerate() {
                let param_name = param.identifier.element.name.clone();

                // Find the parameter in the symbol table
                for (_, symbol) in symbol_table.table.iter_mut() {
                    if symbol.name == param_name {
                        if let Symbol::Parameter { offset, ptr_id } = symbol.symbol {
                            // Skip if pointer is already allocated
                            if ptr_id.is_some() {
                                println!(
                                    "Parameter pointer for symbol {} is not None at function start",
                                    symbol.name
                                );
                                continue;
                            }
                            // Allocate memory for the parameter
                            let ptr = cg.builder.build_alloca(
                                cg.smolpp_types.dynamic_type,
                                format!("alloca_param_{}", param_name).as_str(),
                            )?;

                            // Get function parameter value
                            let param_value = function.get_nth_param(i as u32).unwrap();

                            // Store parameter value in the allocated memory
                            cg.builder.build_store(ptr, param_value)?;

                            // Register the pointer and update the symbol
                            let ptr_id = Some(cg.register_pointer(ptr));
                            (*symbol).symbol = Symbol::Parameter { offset, ptr_id };
                        }
                        break;
                    }
                }
            }

            // Update table
            table_tree.borrow_mut().set_value(symbol_table);
        }

        llvm_from_block(&def.block, cg)?;

        // Return builder to main block
        cg.current_function = cg.main_function;
        cg.builder.position_at_end(cg.current_main_block);
    }

    return Ok(());
}

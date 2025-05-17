use super::{
    llvm_compute_expr, llvm_from_assign, llvm_from_conditional, llvm_from_for_loop,
    llvm_from_return,
};
use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{print::*, smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::{AstNode, Block, Expression, Statement},
    common::{
        diagnostic::{Diagnostic, DiagnosticGravity},
        symbol_table::Symbol,
    },
    typing::{Type, Typeable},
};

pub fn llvm_from_block<'ctx>(
    block: &Block,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let mut error = false;

    // Allocate memory on the stack for each variable
    if let Some(table_tree) = &block.symbol_table {
        let mut symbol_table = table_tree.borrow().get_value();
        for (_, symbol) in symbol_table.table.iter_mut() {
            if let Symbol::Variable { offset, ptr_id } = symbol.symbol {
                if ptr_id.is_some() {
                    println!(
                        "Variable pointer for symbol {} is not None at block start",
                        symbol.name
                    );
                    continue;
                }

                // Allocate memory
                let ptr = cg.builder.build_alloca(
                    cg.smolpp_types.dynamic_type,
                    format!("alloca_var_{}", symbol.name).as_str(),
                )?;

                // Store initial value with correct type
                let val = cg.create_variable(
                    symbol.symbol_type.clone(),
                    cg.context.i64_type().const_zero(),
                )?;
                cg.builder.build_store(ptr, val)?;

                // Register the pointer in the codegen context and update its reference in the symbol table
                let ptr_id = Some(cg.register_pointer(ptr));
                (*symbol).symbol = Symbol::Variable { offset, ptr_id };
            }
        }

        // Update table
        table_tree.borrow_mut().set_value(symbol_table);
    } else {
        panic!("Symbol table not initialized in block")
    }

    // Replace the current symbol table with the block's one
    let old_symbol_table = cg.current_symbol_table.clone();
    cg.current_symbol_table = block.symbol_table.clone();

    for stmt in &block.statements {
        match stmt {
            Statement::Print(expr) => llvm_from_print(expr, cg)?,
            Statement::Assign(assign) => llvm_from_assign(assign, cg)?,
            Statement::Conditional(cond) => llvm_from_conditional(cond, cg)?,
            Statement::For(for_loop) => llvm_from_for_loop(for_loop, cg)?,
            Statement::Return(expr) => llvm_from_return(expr, cg)?,
            Statement::Expr(expr) => {
                let _ = llvm_compute_expr(expr, cg)?;
            }
            Statement::For(_) | Statement::NotImplemented => {
                cg.errors.push(Diagnostic::unimplemented_llvm(stmt));
                error = true;
            }
        }
    }

    // Restore symbol table
    cg.current_symbol_table = old_symbol_table;

    if error {
        return Err(LLVMCodegenError::Unimplemented(String::from("Block")));
    }

    return Ok(());
}

fn llvm_from_print<'ctx>(
    expr: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let expr_value: SmolVar<'ctx> = llvm_compute_expr(expr, cg)?;

    match expr.get_type() {
        Type::None => print_none_value(&expr_value, cg)?,
        Type::Bool => print_bool_value(&expr_value, cg)?,
        Type::Int => print_int_value(&expr_value, cg)?,
        Type::String => print_string_value(&expr_value, cg)?,
        Type::List => print_list_value(&expr_value, cg).map(|_| ())?,
        Type::Weak(_) | Type::Any => print_any_value(&expr_value, cg)?,
        _ => {
            cg.errors.push(Diagnostic::from_localizable_ref(
                expr,
                DiagnosticGravity::Error,
                String::from("UnimplementedLLVM"),
                format!(
                    "Unimplemented llvm for expression {}",
                    expr.get_string_repr()
                ),
            ));
            return Err(LLVMCodegenError::Unimplemented(expr.get_string_repr()));
        }
    };

    return Ok(());
}

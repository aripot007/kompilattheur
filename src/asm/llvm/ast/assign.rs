use inkwell::values::PointerValue;

use crate::{
    asm::{
        codegen::CodeGen,
        internal_global_constants::RuntimeErrorMsg,
        llvm::{
            assert_assignation_type, assert_type, lists, panic::smolpp_panic_with_unreachable,
            smolvar::SmolVar, LLVMCodegenError,
        },
    },
    ast::nodes::{Assign, AstNode, BinOp, Expression, ExpressionKind, Factor, FactorKind},
    common::{
        diagnostic::Diagnostic,
        symbol_table::{get_symbol, Symbol, SymbolTableElement},
    },
    typing::Type,
};
use inkwell::AddressSpace;

use super::llvm_compute_expr;

pub fn llvm_from_assign<'ctx>(
    assign: &Assign,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let expr_value: SmolVar<'ctx> = llvm_compute_expr(&assign.value, cg)?;

    let dest_ptr_opt = compute_destination_ptr(&assign.destination, cg)?;

    // Store assignation result if destination pointer is not None
    if let Some(dest_ptr) = dest_ptr_opt {
        let dest_var =
            cg.builder
                .build_load(cg.smolpp_types.dynamic_type, dest_ptr, "load_dest_value")?;

        assert_assignation_type(&dest_var.into_struct_value(), &expr_value, cg)?;

        // Store the result
        cg.builder.build_store(dest_ptr, expr_value)?;
    } else {
        // Emit warning for discarded assignation
        cg.warnings
            .push(Diagnostic::discarded_assign_result(assign));
    }

    return Ok(());
}

/// Compute the destination pointer from an expression
fn compute_destination_ptr<'ctx>(
    dest_expr: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<Option<PointerValue<'ctx>>, LLVMCodegenError> {
    match &dest_expr.kind {
        ExpressionKind::Factor(factor) => return factor_to_dest_ptr(factor, cg),
        ExpressionKind::UNOP(_, _) => {
            cg.errors
                .push(Diagnostic::invalid_destination_expr(dest_expr));
            return Err(LLVMCodegenError::InvalidDestination(
                dest_expr.get_string_repr(),
            ));
        }
        ExpressionKind::BINOP(expr1, BinOp::ACCESS, expr2) => {
            return access_to_ptr(expr1, expr2, cg);
        }
        _ => {
            cg.errors.push(Diagnostic::unimplemented_llvm(dest_expr));
            return Err(LLVMCodegenError::Unimplemented(dest_expr.get_string_repr()));
        }
    };
}

/// Compute a destination pointer from a factor
fn factor_to_dest_ptr<'ctx>(
    factor: &Factor,
    cg: &mut CodeGen<'ctx>,
) -> Result<Option<PointerValue<'ctx>>, LLVMCodegenError> {
    match &factor.kind {
        FactorKind::Expr(expression) => return compute_destination_ptr(expression, cg),
        FactorKind::Identifier(fe) => {
            let st = cg
                .current_symbol_table
                .clone()
                .expect("Symbol table not initialized while generating llvm");
            let symbol: SymbolTableElement = get_symbol(&st, &fe.element.id).expect(
                format!(
                    "Symbol {} ({}) not registered",
                    fe.element.name, fe.element.id
                )
                .as_str(),
            );

            let ptr: PointerValue<'ctx> = match symbol.symbol {
                Symbol::Variable { ptr_id, .. } => cg.get_pointer(ptr_id.unwrap()).unwrap().clone(),
                Symbol::Parameter { ptr_id, .. } => {
                    cg.get_pointer(ptr_id.unwrap()).unwrap().clone()
                }
                _ => panic!(),
            };

            // TODO(Baptiste): assert type
            /*
            let smalvar = cg
                .builder
                .build_load(cg.smolpp_types.dynamic_type, ptr, "var struct")?
                .into_struct_value();
            assert_type(
                Type::List,
                &smalvar,
                cg,
                Some("Expected list for access".into()),
            )?;
            */

            return Ok(Some(ptr));
        }

        FactorKind::List(_) | FactorKind::String(_) | FactorKind::Call { .. } => return Ok(None),

        _ => {
            cg.errors.push(Diagnostic::invalid_destination_expr(factor));
            return Err(LLVMCodegenError::InvalidDestination(
                factor.get_string_repr(),
            ));
        }
    }
}

fn access_to_ptr<'ctx>(
    expr1: &Expression,
    expr2: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<Option<PointerValue<'ctx>>, LLVMCodegenError> {
    // Compute the left-hand side of the access operation (should be a list)
    let base_value = compute_destination_ptr(expr1, cg)?;
    let base_value = match base_value {
        None => return Ok(None),
        Some(e) => e,
    };
    let base_value = cg
        .builder
        .build_load(cg.smolpp_types.dynamic_type, base_value, "list var")?
        .into_struct_value();

    // Evaluate the index expression
    let index_value = super::llvm_compute_expr(expr2, cg)?;

    // Check if the index is an integer
    if let Some(index_type) = &expr2.expr_type {
        if *index_type != Type::Int {
            cg.errors.push(Diagnostic::invalid_destination_expr(expr2));
            return Err(LLVMCodegenError::InvalidDestination(format!(
                "List index must be an integer, got {}",
                index_type
            )));
        }
    }

    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    // Extract the list pointer from the base value
    let list_ptr = cg.get_variable_value(base_value)?.into_int_value();
    let list_ptr = cg
        .builder
        .build_int_to_ptr(list_ptr, ptr_type, "list_ptr")?;

    // Get the list struct
    let list_struct = cg
        .builder
        .build_load(cg.smolpp_types.list_type, list_ptr, "list_struct")?
        .into_struct_value();

    // Get the array pointer from the list struct
    let array_ptr = cg.get_list_array_ptr(list_struct)?;

    // Get the index as an i32 value for GEP instruction
    let index_int_value = cg.get_variable_value(index_value)?;
    let index_i32 = cg.builder.build_int_truncate(
        index_int_value.into_int_value(),
        cg.context.i32_type(),
        "index_i32",
    )?;

    // Add bounds check
    let list_length = cg.get_list_length(list_struct)?;
    let list_length_i32 =
        cg.builder
            .build_int_truncate(list_length, cg.context.i32_type(), "length_i32")?;

    // Create basic blocks for the bounds check
    let current_function = cg.builder.get_insert_block().unwrap().get_parent().unwrap();
    let in_bounds_block = cg.context.append_basic_block(current_function, "in_bounds");
    let out_of_bounds_block = cg
        .context
        .append_basic_block(current_function, "out_of_bounds");
    let continue_block = cg.context.append_basic_block(current_function, "continue");

    // Check if index >= 0 && index < length
    let is_negative = cg.builder.build_int_compare(
        inkwell::IntPredicate::SLT,
        index_i32,
        cg.context.i32_type().const_zero(),
        "is_negative",
    )?;

    let is_too_large = cg.builder.build_int_compare(
        inkwell::IntPredicate::SGE,
        index_i32,
        list_length_i32,
        "is_too_large",
    )?;

    let is_out_of_bounds = cg
        .builder
        .build_or(is_negative, is_too_large, "is_out_of_bounds")?;

    cg.builder
        .build_conditional_branch(is_out_of_bounds, out_of_bounds_block, in_bounds_block)?;

    // Handle out of bounds error
    cg.builder.position_at_end(out_of_bounds_block);

    // Create error message
    let error_msg = cg.context.const_string(
        format!(
            "IndexError: index {} out of bounds for list of length {}",
            expr2.get_string_repr(),
            list_length_i32
        )
        .as_bytes(),
        true,
    );

    // Call panic function
    smolpp_panic_with_unreachable(cg, RuntimeErrorMsg::TypeError, &[error_msg.into()])?;

    // Handle in-bounds access
    cg.builder.position_at_end(in_bounds_block);

    // Get the element pointer using GEP
    let elt_ptr = unsafe {
        cg.builder.build_gep(
            cg.smolpp_types.dynamic_type,
            array_ptr,
            &[index_i32],
            "list_elt_ptr",
        )
    }?;

    cg.builder.build_unconditional_branch(continue_block)?;

    // Continue block
    cg.builder.position_at_end(continue_block);

    return Ok(Some(elt_ptr));
}

use inkwell::values::PointerValue;

use crate::{
    asm::{
        codegen::CodeGen,
        internal_global_constants::RuntimeErrorMsg,
        llvm::{assert_type, panic::smolpp_panic_with_unreachable, LLVMCodegenError},
    },
    ast::nodes::{AstNode, BinOp, Expression, ExpressionKind, Factor, FactorKind},
    common::{
        diagnostic::Diagnostic,
        symbol_table::{get_symbol, Symbol, SymbolTableElement},
    },
    typing::Type,
};
use inkwell::AddressSpace;

use super::llvm_compute_factor;

pub enum MemoryPtr<'ctx> {
    Storable(PointerValue<'ctx>),
    ReadOnly(PointerValue<'ctx>),
}

/// Compute the destination pointer from an expression
pub fn compute_destination_ptr<'ctx>(
    dest_expr: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<MemoryPtr<'ctx>, LLVMCodegenError> {
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
) -> Result<MemoryPtr<'ctx>, LLVMCodegenError> {
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

            return Ok(MemoryPtr::Storable(ptr));
        }

        FactorKind::List(_) | FactorKind::String(_) | FactorKind::Call { .. } => {
            let smolvar = llvm_compute_factor(factor, cg)?;
            let ptr = cg
                .builder
                .build_alloca(cg.smolpp_types.dynamic_type, "value")?;
            cg.builder.build_store(ptr, smolvar)?;
            return Ok(MemoryPtr::ReadOnly(ptr));
        }

        _ => {
            cg.errors.push(Diagnostic::invalid_destination_expr(factor));
            return Err(LLVMCodegenError::InvalidDestination(
                factor.get_string_repr(),
            ));
        }
    }
}

pub fn access_to_ptr<'ctx>(
    expr1: &Expression,
    expr2: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<MemoryPtr<'ctx>, LLVMCodegenError> {
    // Compute the left-hand side of the access operation (should be a list)
    let base_mem_ptr = compute_destination_ptr(expr1, cg)?;
    let mut read_only = false;
    let base_value = match base_mem_ptr {
        MemoryPtr::ReadOnly(e) => {
            read_only = true;
            e
        }
        MemoryPtr::Storable(e) => e,
    };
    let base_value = cg
        .builder
        .build_load(cg.smolpp_types.dynamic_type, base_value, "list var")?
        .into_struct_value();

    // TODO: optimize if not Weak
    assert_type::assert_type(
        Type::List,
        &base_value,
        cg,
        Some("Expected list for access".into()),
        Some(expr1),
    )?;

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
    let array_ptr = cg.build_get_list_array_ptr(list_struct)?;

    // Get the index as an i32 value for GEP instruction
    let index_int_value = cg.get_variable_value(index_value)?;
    let index_i32 = cg.builder.build_int_truncate(
        index_int_value.into_int_value(),
        cg.context.i32_type(),
        "index_i32",
    )?;

    // Add bounds check
    let list_length = cg.build_get_list_length(list_struct)?;
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

    // Call panic function
    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::IndexOutOfBound,
        &[index_i32.into(), list_length_i32.into()],
        Some(expr2),
    )?;

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

    if read_only {
        return Ok(MemoryPtr::ReadOnly(elt_ptr));
    }

    return Ok(MemoryPtr::Storable(elt_ptr));
}

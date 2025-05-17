use inkwell::values::PointerValue;

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{assert_assignation_type, smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::{Assign, AstNode, BinOp, Expression, ExpressionKind, Factor, FactorKind},
    common::{
        diagnostic::Diagnostic,
        symbol_table::{get_symbol, Symbol, SymbolTableElement},
    },
};

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
        ExpressionKind::BINOP(expr1, binop, expr2) => {
            return access_to_ptr(expr1, binop, expr2, cg)
        }
        ExpressionKind::NotImplemented => {
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
            let symbol: SymbolTableElement = get_symbol(st, &fe.element.id).1.expect(
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
    binop: &BinOp,
    expr2: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<Option<PointerValue<'ctx>>, LLVMCodegenError> {
    // Only handle access operations
    if !matches!(*binop, BinOp::ACCESS) {
        cg.errors.push(Diagnostic::invalid_destination_expr(expr1));
        return Err(LLVMCodegenError::InvalidDestination(format!(
            "Invalid binary operation for destination: {}",
            expr1.get_string_repr()
        )));
    }

    // TODO: Implement pointer access for array indexing and struct field access
    let error_msg = format!(
        "Access operation not implemented: {} {} {}",
        expr1.get_string_repr(),
        binop,
        expr2.get_string_repr()
    );

    // Add diagnostic
    cg.errors.push(Diagnostic::unimplemented_llvm(expr1));

    Err(LLVMCodegenError::Unimplemented(error_msg))
}

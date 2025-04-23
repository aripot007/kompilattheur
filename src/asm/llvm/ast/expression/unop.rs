use crate::{asm::{codegen::CodeGen, llvm::{assert_type, smolvar::SmolVar, LLVMCodegenError}}, ast::nodes::{Expression, UnOp}, typing::Type};

use super::llvm_compute_expr;


pub fn llvm_compute_unop<'ctx>(op: &UnOp, expr: &Expression, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    
    let val = llvm_compute_expr(expr, cg)?;

    match (op, &expr.expr_type) {
        (UnOp::NOT, Some(Type::Bool)) => llvm_compute_not_unchecked(val, cg),
        (UnOp::NOT, _) => llvm_compute_not(val, cg),
        (UnOp::NEG, Some(Type::Int)) => llvm_compute_neg_unchecked(val, cg),
        (UnOp::NEG, _) => llvm_compute_neg(val, cg),
    }
}

/// Compute the NOT operation for a given variable
fn llvm_compute_not<'ctx>(val: SmolVar<'ctx>, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Bool, &val, cg, None)?;
    return llvm_compute_not_unchecked(val, cg);
}

/// Compute the NOT operation for a given variable, without type checking
fn llvm_compute_not_unchecked<'ctx>(val: SmolVar<'ctx>, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val_field = cg.get_variable_value(val)?.into_int_value();
    let new_value = cg.builder.build_not(val_field, "smolvar_not")?;
    return cg.set_variable_value(val, new_value);
}

/// Compute the NEG operation for a given variable
fn llvm_compute_neg<'ctx>(val: SmolVar<'ctx>, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &val, cg, None)?;
    return llvm_compute_neg_unchecked(val, cg);
}

/// Compute the NEG operation for a given variable, without type checking
fn llvm_compute_neg_unchecked<'ctx>(val: SmolVar<'ctx>, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val_field = cg.get_variable_value(val)?.into_int_value();
    let new_value = cg.builder.build_int_neg(val_field, "smolvar_neg")?;
    return cg.set_variable_value(val, new_value);
}

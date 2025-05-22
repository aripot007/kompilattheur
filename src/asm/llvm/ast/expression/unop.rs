use crate::{
    asm::{
        codegen::CodeGen,
        get_internal_func,
        llvm::{assert_type::assert_type, smolvar::SmolVar, LLVMCodegenError},
        InternalFuctions,
    },
    ast::nodes::{Expression, UnOp},
    common::localizable::Localizable,
    typing::Type,
};

use super::llvm_compute_expr;

pub fn llvm_compute_unop<'ctx>(
    op: &UnOp,
    expr: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val = llvm_compute_expr(expr, cg)?;

    match (op, &expr.expr_type) {
        (UnOp::NOT, Some(Type::Bool)) => llvm_compute_not_unchecked(val, cg),
        (UnOp::NOT, _) => llvm_compute_not(val, cg),
        (UnOp::NEG, Some(Type::Int)) => llvm_compute_neg_unchecked(val, cg),
        (UnOp::NEG, _) => llvm_compute_neg(val, cg, Some(expr)),
    }
}

/// Compute the NOT operation for a given variable
fn llvm_compute_not<'ctx>(
    val: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // Cast e1_value to bool using the bool_cast_internal_function
    let call_boolean_llvm_value = cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::BoolCast),
        &[val.into()],
        "not_bool_cast_call",
    )?;

    let boolean_llvm_value = call_boolean_llvm_value
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let not_boolean_llvm_value = cg.builder.build_not(boolean_llvm_value, "not not")?;

    let result =
        cg.builder
            .build_int_cast(not_boolean_llvm_value, cg.context.i64_type(), "not cast")?;

    cg.create_variable(Type::Bool, result)
}

/// Compute the NOT operation for a given variable, without type checking
fn llvm_compute_not_unchecked<'ctx>(
    val: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val_field = cg.get_variable_value(val)?.into_int_value();
    let val_as_bool =
        cg.builder
            .build_int_cast(val_field, cg.context.bool_type(), "val_as_bool")?;
    let new_value = cg.builder.build_not(val_as_bool, "smolvar_not")?;
    let new_value = cg
        .builder
        .build_int_cast(new_value, cg.context.i64_type(), "val_as_int")?;
    return cg.set_variable_value(val, new_value);
}

/// Compute the NEG operation for a given variable
fn llvm_compute_neg<'ctx, T>(
    val: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
    loc: Option<T>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError>
where
    T: Localizable,
{
    assert_type(Type::Int, &val, cg, loc)?;
    return llvm_compute_neg_unchecked(val, cg);
}

/// Compute the NEG operation for a given variable, without type checking
fn llvm_compute_neg_unchecked<'ctx>(
    val: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val_field = cg.get_variable_value(val)?.into_int_value();
    let new_value = cg.builder.build_int_neg(val_field, "smolvar_neg")?;
    return cg.set_variable_value(val, new_value);
}

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{access_to_ptr, smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::{BinOp, Expression},
    common::diagnostic::Diagnostic,
    typing::{Type, Typeable},
};

use super::{
    compare_generic_values, compare_int_bool_values, compare_list_values, compare_none_values, compare_string_values, compute_add_unchecked, compute_div_unchecked, compute_mod_unchecked, compute_mult_unchecked, compute_sub_unchecked, llvm_compute_and_or, llvm_compute_expr
};

use super::super::access::MemoryPtr;

pub fn llvm_compute_binop<'ctx>(
    e1: &Expression,
    op: &BinOp,
    e2: &Expression,
    cg: &mut CodeGen<'ctx>,
    root: &Expression,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    match op {
        BinOp::AND | BinOp::OR => return llvm_compute_and_or(e1, op, e2, cg),

        BinOp::LESS
        | BinOp::LESSEQ
        | BinOp::GREATER
        | BinOp::GREATEREQ
        | BinOp::EQ
        | BinOp::NEQ => return llvm_compute_comparison(e1, op, e2, cg),

        BinOp::MULT | BinOp::DIV | BinOp::MOD | BinOp::SUB => {
            return llvm_compute_arithmetic(e1, op, e2, cg, root)
        }

        // TODO : Remove when generic add is implemented
        BinOp::ADD if e1.get_type() == &Type::Int && e2.get_type() == &Type::Int => {
            return llvm_compute_arithmetic(e1, op, e2, cg, root)
        }

        BinOp::ADD => {
            cg.errors.push(Diagnostic::unimplemented_llvm(root));
            Err(LLVMCodegenError::Unimplemented(format!(
                "ADD operation not implemented yet"
            )))
        }

        BinOp::ACCESS => {
            let ptr = access_to_ptr(e1, e2, cg)?;
            let ptr = match ptr {
                MemoryPtr::Storable(e) => e,
                MemoryPtr::ReadOnly(e) => e,
            };
            let smolvar = cg
                .builder
                .build_load(cg.smolpp_types.dynamic_type, ptr, "smolvar")?
                .into_struct_value();
            Ok(smolvar)
        }
    }
}

fn llvm_compute_arithmetic<'ctx>(
    e1: &Expression,
    op: &BinOp,
    e2: &Expression,
    cg: &mut CodeGen<'ctx>,
    root: &Expression,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = llvm_compute_expr(e1, cg)?;
    let val2 = llvm_compute_expr(e2, cg)?;

    if e1.get_type() == &Type::Int && e2.get_type() == &Type::Int {
        match op {
            BinOp::MULT => compute_mult_unchecked(val1, val2, cg),
            BinOp::DIV => compute_div_unchecked(val1, val2, cg),
            BinOp::MOD => compute_mod_unchecked(val1, val2, cg),
            BinOp::SUB => compute_sub_unchecked(val1, val2, cg),
            BinOp::ADD => compute_add_unchecked(val1, val2, cg),
            _ => panic!("Trying to compute arithmetic with a {} operation", op),
        }
    } else {
        // Weak types or non arithmetic types, not implemented yet
        cg.errors.push(Diagnostic::unimplemented_llvm(root));
        return Err(LLVMCodegenError::Unimplemented(String::from(
            "Arithmetic for dynamically typed expressions is not implemented yet",
        )));
    }
}

fn llvm_compute_comparison<'ctx>(
    e1: &Expression,
    op: &BinOp,
    e2: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1: inkwell::values::StructValue<'_> = llvm_compute_expr(e1, cg)?;
    let val2 = llvm_compute_expr(e2, cg)?;

    match (e1.get_type(), e2.get_type()) {
        (Type::Int, Type::Int) => compare_int_bool_values(val1, val2, op.clone(), cg),
        (Type::String, Type::String) => compare_string_values(val1, val2, op.clone(), cg),
        (Type::None, Type::None) => compare_none_values(val1, val2, op.clone(), cg),
        (Type::Bool, Type::Bool) => compare_int_bool_values(val1, val2, op.clone(), cg),
        (Type::List, Type::List) => compare_list_values(val1, val2, op.clone(), cg),
        (Type::Bool, Type::Int) => compare_int_bool_values(val1, val2, op.clone(), cg),
        (Type::Int, Type::Bool) => compare_int_bool_values(val1, val2, op.clone(), cg),
        _ => compare_generic_values(val1, val2, op.clone(), cg),
    }
}

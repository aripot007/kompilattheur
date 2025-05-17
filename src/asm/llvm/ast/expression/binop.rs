use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::{BinOp, Expression},
    common::diagnostic::Diagnostic,
    typing::{Type, Typeable},
};

use super::{
    compare_boolean_values, compare_generic_values, compare_int_values, compare_list_values,
    compare_none_values, compare_string_values, compute_add_unchecked, compute_div_unchecked,
    compute_mod_unchecked, compute_mult_unchecked, compute_sub_unchecked, llvm_compute_expr,
};

pub fn llvm_compute_binop<'ctx>(
    e1: &Expression,
    op: &BinOp,
    e2: &Expression,
    cg: &mut CodeGen<'ctx>,
    root: &Expression,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    match op {
        BinOp::AND
        | BinOp::OR
        | BinOp::LESS
        | BinOp::LESSEQ
        | BinOp::GREATER
        | BinOp::GREATEREQ
        | BinOp::EQ
        | BinOp::NEQ => return llvm_compute_comparison(e1, op, e2, cg, root),

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
            cg.errors.push(Diagnostic::unimplemented_llvm(root));
            Err(LLVMCodegenError::Unimplemented(String::from(
                "ACCESS operation not implemented yet",
            )))
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
    root: &Expression,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = llvm_compute_expr(e1, cg)?;
    let val2 = llvm_compute_expr(e2, cg)?;

    if e1.get_type() == &Type::Int && e2.get_type() == &Type::Int {
        compare_int_values(val1, val2, op.clone(), cg)
    } else if e1.get_type() == &Type::String && e2.get_type() == &Type::String {
        compare_string_values(val1, val2, op.clone(), cg)
    } else if e1.get_type() == &Type::None && e2.get_type() == &Type::None {
        compare_none_values(val1, val2, op.clone(), cg)
    } else if e1.get_type() == &Type::Bool && e2.get_type() == &Type::Bool {
        compare_boolean_values(val1, val2, op.clone(), cg)
    } else if e1.get_type() == &Type::List && e2.get_type() == &Type::List {
        compare_list_values(val1, val2, op.clone(), cg)
    } else {
        compare_generic_values(val1, val2, op.clone(), cg)
    }
}

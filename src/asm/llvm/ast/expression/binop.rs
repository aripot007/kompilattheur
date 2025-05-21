use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{
            access_to_ptr,
            assert_type::assert_dyn_type,
            ast::expression::{
                compute_add_string, compute_div, compute_mod, compute_mult, compute_sub,
            },
            smolvar::SmolVar,
            LLVMCodegenError,
        },
    },
    ast::nodes::{BinOp, Expression},
    typing::{Type, Typeable},
};

use super::{
    compare_generic_values, compare_int_bool_values, compare_list_values, compare_none_values,
    compare_string_values, compute_add, compute_add_generic, compute_add_list, compute_add_range,
    compute_add_unchecked, compute_div_unchecked, compute_mod_unchecked, compute_mult_unchecked,
    compute_sub_unchecked, llvm_compute_and_or, llvm_compute_expr,
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
            return llvm_compute_arithmetic(e1, op, e2, cg)
        }

        BinOp::ADD => return llvm_compute_add(e1, e2, cg),

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
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = llvm_compute_expr(e1, cg)?;
    let val2 = llvm_compute_expr(e2, cg)?;

    if e1.get_type() == &Type::Int && e2.get_type() == &Type::Int {
        match op {
            BinOp::MULT => compute_mult_unchecked(val1, val2, cg),
            BinOp::DIV => compute_div_unchecked(val1, val2, cg),
            BinOp::MOD => compute_mod_unchecked(val1, val2, cg),
            BinOp::SUB => compute_sub_unchecked(val1, val2, cg),
            _ => panic!("Trying to compute arithmetic with a {} operation", op),
        }
    } else {
        assert!(e1.get_type().is_compatible(Type::Int) && e2.get_type().is_compatible(Type::Int));

        match op {
            BinOp::MULT => compute_mult(val1, val2, cg),
            BinOp::DIV => compute_div(val1, val2, cg),
            BinOp::MOD => compute_mod(val1, val2, cg),
            BinOp::SUB => compute_sub(val1, val2, cg),
            _ => panic!("Trying to compute arithmetic with a {} operation", op),
        }
    }
}

fn llvm_compute_add<'ctx>(
    e1: &Expression,
    e2: &Expression,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = llvm_compute_expr(e1, cg)?;
    let val2 = llvm_compute_expr(e2, cg)?;

    match (e1.get_type(), e2.get_type()) {
        (Type::Int, Type::Int) => return compute_add_unchecked(val1, val2, cg),
        (Type::Int, t) | (t, Type::Int) => {
            assert!(t.is_compatible(Type::Int));
            return compute_add(val1, val2, cg);
        }
        (Type::String, Type::String) => return compute_add_string(val1, val2, cg),
        (Type::String, t) | (t, Type::String) => {
            assert!(t.is_compatible(Type::String));
            assert_dyn_type(&val1, &val2, cg)?;
            return compute_add_string(val1, val2, cg);
        }
        (Type::List, Type::List) => return compute_add_list(val1, val2, cg),
        (Type::List, t) | (t, Type::List) => {
            assert!(t.is_compatible(Type::List));
            assert_dyn_type(&val1, &val2, cg)?;
            return compute_add_list(val1, val2, cg);
        }
        (Type::Range, Type::Range) => return compute_add_range(val1, val2, cg),
        (Type::Range, t) | (t, Type::Range) => {
            assert!(t.is_compatible(Type::Range));
            assert_dyn_type(&val1, &val2, cg)?;
            return compute_add_range(val1, val2, cg);
        }
        _ => return compute_add_generic(val1, val2, cg),
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

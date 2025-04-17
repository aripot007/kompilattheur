
use crate::asm::llvm::smolvar::SmolVar;
use crate::ast::nodes::{AstNode, FactorKind};
use crate::{asm::codegen::CodeGen, ast::nodes::Factor, common::diagnostic::Diagnostic, typing::Type};
use crate::asm::llvm::LLVMCodegenError;

use super::llvm_compute_expr;

pub fn llvm_compute_factor<'ctx>(factor: &Factor, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {

    match &factor.kind {
        FactorKind::String(file_element) => return llvm_compute_string_value(&file_element.element, cg),
        FactorKind::Integer(file_element) => return llvm_compute_int_value(file_element.element, cg),
        FactorKind::True(_) => return llvm_compute_bool_value(true, cg),
        FactorKind::False(_) => return llvm_compute_bool_value(false, cg),
        FactorKind::None(_) => return llvm_compute_none_value(cg),
        FactorKind::Expr(expr) => return llvm_compute_expr(expr, cg),
        FactorKind::List(_) => return llvm_compute_list_value(cg),
        FactorKind::Identifier(_)
        | FactorKind::Call { identifier: _, args: _, localization: _ } => (),
    }

    cg.errors.push(Diagnostic::unimplemented_llvm(factor));
   
    return Err(LLVMCodegenError::Unimplemented(factor.get_string_repr()));
}

fn llvm_compute_string_value<'ctx>(s: &String, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    
    let str_const_ptr = cg.builder.build_global_string_ptr(&s, "string_const")?;

    return Ok(cg.create_variable(Type::String, str_const_ptr.as_pointer_value()));
}

fn llvm_compute_int_value<'ctx>(value: u64, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let int_const = cg.context.i64_type().const_int(value, true);

    return Ok(cg.create_variable(Type::Int, int_const));
}

fn llvm_compute_bool_value<'ctx>(value: bool, cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {

    let int_const = cg.context.i64_type().const_int(value as u64, false);

    return Ok(cg.create_variable(Type::Bool, int_const));
}

fn llvm_compute_none_value<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {

    let val = cg.context.i64_type().const_zero();

    return Ok(cg.create_variable(Type::None, val));
}

fn llvm_compute_list_value<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {

    let val = cg.context.i64_type().const_zero();

    return Ok(cg.create_variable(Type::List, val));
}


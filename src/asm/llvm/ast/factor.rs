use inkwell::values::StructValue;

use crate::ast::nodes::FactorKind;
use crate::{asm::codegen::CodeGen, ast::nodes::Factor, common::diagnostic::Diagnostic, typing::Type};
use crate::asm::llvm::LLVMCodegenError;

use super::llvm_compute_expr;

/// Create a constant variable with the given type and value
macro_rules! const_variable {
    ($cg: ident, $t: expr, $v: expr) => {
        {
            let var_type_discr = $t.get_discriminant();
            let var_type_discr_val = $cg.context.i8_type().const_int(var_type_discr as u64, false);
            $cg.smolpp_types.dynamic_type.const_named_struct(
                &[
                    var_type_discr_val.into(),
                    $v.into(),
                ]
            )
        }
    };
}

pub fn llvm_compute_factor<'ctx>(factor: &Factor, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

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
   
    return Err(());
}

fn llvm_compute_string_value<'ctx>(s: &String, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {
    
    let str_const_ptr = cg.builder.build_global_string_ptr(&s, "string_const").unwrap();

    return Ok(const_variable!(cg, Type::String, str_const_ptr.as_pointer_value()));
}

fn llvm_compute_int_value<'ctx>(value: u64, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    let int_const = cg.context.i64_type().const_int(value, false);

    return Ok(const_variable!(cg, Type::Int, int_const));
}

fn llvm_compute_bool_value<'ctx>(value: bool, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    let int_const = cg.context.i64_type().const_int(value as u64, false);

    return Ok(const_variable!(cg, Type::Bool, int_const));
}

fn llvm_compute_none_value<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    let val = cg.context.i64_type().const_zero();

    return Ok(const_variable!(cg, Type::None, val));
}

fn llvm_compute_list_value<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    let val = cg.context.i64_type().const_zero();

    return Ok(const_variable!(cg, Type::List, val));
}


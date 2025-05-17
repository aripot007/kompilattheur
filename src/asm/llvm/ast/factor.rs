use crate::asm::llvm::smolvar::SmolVar;
use crate::asm::llvm::LLVMCodegenError;
use crate::ast::nodes::Expression;
use crate::ast::nodes::FactorKind;
use crate::common::symbol_table::{get_symbol, Symbol, SymbolTableElement};
use crate::common::types::{FileElement, IdToken};
use crate::{asm::codegen::CodeGen, ast::nodes::Factor, typing::Type};

use super::llvm_compute_expr;

pub fn llvm_compute_factor<'ctx>(
    factor: &Factor,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    match &factor.kind {
        FactorKind::String(file_element) => {
            return llvm_compute_string_value(&file_element.element, cg)
        }
        FactorKind::Integer(file_element) => {
            return llvm_compute_int_value(file_element.element, cg)
        }
        FactorKind::True(_) => return llvm_compute_bool_value(true, cg),
        FactorKind::False(_) => return llvm_compute_bool_value(false, cg),
        FactorKind::None(_) => return llvm_compute_none_value(cg),
        FactorKind::Expr(expr) => return llvm_compute_expr(expr, cg),
        FactorKind::Identifier(FileElement {
            element: id_token, ..
        }) => return llvm_compute_identifier_value(id_token, cg),
        FactorKind::List(_) => return llvm_compute_list_value(cg),
        FactorKind::Call {
            identifier: id,
            args: args,
            localization: localization,
        } => return llvm_compute_function_call(id, args, localization, cg),
    }
}

fn llvm_compute_string_value<'ctx>(
    s: &String,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let str_const_ptr = cg.builder.build_global_string_ptr(&s, "string_const")?;

    return cg.create_variable(Type::String, str_const_ptr.as_pointer_value());
}

fn llvm_compute_int_value<'ctx>(
    value: u64,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let int_const = cg.context.i64_type().const_int(value, true);

    return cg.create_variable(Type::Int, int_const);
}

fn llvm_compute_bool_value<'ctx>(
    value: bool,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let int_const = cg.context.bool_type().const_int(value as u64, false);

    return cg.create_variable(Type::Bool, int_const);
}

fn llvm_compute_none_value<'ctx>(
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val = cg.context.i64_type().const_zero();

    return cg.create_variable(Type::None, val);
}

fn llvm_compute_list_value<'ctx>(
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val = cg.context.i64_type().const_zero();

    return cg.create_variable(Type::List, val);
}

fn llvm_compute_identifier_value<'ctx>(
    id_token: &IdToken,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let symbol: SymbolTableElement =
        get_symbol(cg.current_symbol_table.clone().unwrap(), &id_token.id)
            .1
            .unwrap();
    let val_ptr_id = match symbol.symbol {
        Symbol::Variable { ptr_id, .. } => ptr_id.unwrap(),
        Symbol::Parameter { .. } => todo!("Add param ptr_id"),
        _ => panic!(),
    };
    let val_ptr = *cg.get_pointer(val_ptr_id).unwrap();

    let val = cg.builder.build_load(
        cg.smolpp_types.dynamic_type,
        val_ptr,
        format!("load_{}", id_token.name).as_str(),
    )?;

    return Ok(val.into_struct_value());
}

fn llvm_compute_function_call<'ctx>(
    identifier: &IdToken,
    args: &Vec<Expression>,
    localization: &FileElement<bool>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // For now, return a placeholder None value
    // In a full implementation, this would need to actually call the function
    let val = cg.context.i64_type().const_zero();

    // Return a SmolVar with None type
    return cg.create_variable(Type::None, val);
}

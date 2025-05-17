use crate::asm::llvm::smolvar::SmolVar;
use crate::asm::llvm::LLVMCodegenError;
use crate::ast::nodes::Expression;
use crate::ast::nodes::FactorKind;
use crate::common::symbol_table::{get_symbol, Symbol, SymbolTableElement};
use crate::common::types::{FileElement, IdToken};
use crate::{asm::codegen::CodeGen, ast::nodes::Factor, typing::Type};
use inkwell::values::{BasicMetadataValueEnum, StructValue};
use inkwell::AddressSpace;

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
        FactorKind::List(values) => return llvm_compute_list_value(values, cg),
        FactorKind::Call {
            identifier,
            args,
            localization: _,
        } => return llvm_compute_function_call(identifier, args, cg),
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
    values: &Vec<Expression>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let capa = cg.context.i64_type().const_int(values.len() as u64, false);
    let (val, list_struct_ptr) = cg.build_list_variable(capa, false)?;

    // Update len
    let len_ptr =
        cg.builder
            .build_struct_gep(cg.smolpp_types.list_type, list_struct_ptr, 0, "len_ptr")?;
    cg.builder.build_store(len_ptr, capa)?;

    let array_ptr_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        list_struct_ptr,
        2,
        "array_ptr_ptr",
    )?;
    let array_ptr = cg.builder.build_load(
        cg.context.ptr_type(AddressSpace::default()),
        array_ptr_ptr,
        "array_ptr",
    )?;

    for (i, value) in values.iter().enumerate() {
        let list_index = cg.context.i32_type().const_int(i as u64, false);
        let list_elt = llvm_compute_expr(value, cg)?;
        let elt_ptr = unsafe {
            cg.builder.build_gep(
                cg.smolpp_types.dynamic_type,
                array_ptr.into_pointer_value(),
                &[list_index],
                "elt_ptr",
            )
        }?;
        cg.builder.build_store(elt_ptr, list_elt)?;
    }

    return Ok(val);
}

fn llvm_compute_identifier_value<'ctx>(
    id_token: &IdToken,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let symbol_table = cg.current_symbol_table.as_ref().unwrap();
    let symbol: SymbolTableElement = get_symbol(symbol_table, &id_token.id).unwrap();
    let val_ptr_id = match symbol.symbol {
        Symbol::Variable { ptr_id, .. } => ptr_id.unwrap(),
        Symbol::Parameter { ptr_id, .. } => ptr_id.unwrap(),
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
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let opt_function_value = cg
        .module
        .get_function(&format!("__smolpp_user_f_{}", identifier.name).as_str());
    let function_value;
    match opt_function_value {
        Some(e) => function_value = e,
        None => panic!(
            "Error while generating LLVM try call function {} which doesn't exist.",
            identifier.name
        ),
    }

    let mut computed_args: Vec<BasicMetadataValueEnum<'_>> = vec![];
    for arg in args {
        computed_args.push(llvm_compute_expr(arg, cg)?.into());
    }

    let call_site_value = cg.builder.build_call(
        function_value,
        &computed_args,
        format!("function_call_{}", identifier.name).as_str(),
    )?;

    let return_value = call_site_value.try_as_basic_value().left().unwrap();

    let struct_value = StructValue::try_from(return_value).unwrap();

    return Ok(struct_value);
}

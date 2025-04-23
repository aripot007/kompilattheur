use inkwell::values::{IntValue, StructValue};

use crate::{asm::codegen::CodeGen, ast::nodes::BinOp};

/// Compare two Integer with the given operation
pub fn compare_int_values<'ctx>(value1: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> IntValue<'ctx> {
    todo!()
}

/// Compare two String with the given operation
pub fn compare_string_values<'ctx>(value1: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> IntValue<'ctx> {
    todo!()
}

/// Compare two None with the given operation
pub fn compare_none_values<'ctx>(value1: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> IntValue<'ctx> {
    todo!()
}

/// Compare two Boolean with the given operation
pub fn compare_boolean_values<'ctx>(value1: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> IntValue<'ctx> {
    todo!()
}

/// Compare two List with the given operation
pub fn compare_list_values<'ctx>(value1: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> IntValue<'ctx> {
    todo!()
}

/// Compare two generic values with the given operation
pub fn compare_generic_values<'ctx>(value1: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> IntValue<'ctx> {
    todo!()
}
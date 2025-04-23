use crate::{asm::{codegen::CodeGen, llvm::{assert_type, smolvar::SmolVar, LLVMCodegenError}}, ast::nodes::BinOp, typing::Type};
/// Compare two Integer with the given operation
pub fn compare_int_values<'ctx>(value1: &SmolVar<'ctx>, value2: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = cg.get_variable_value(value1)?.into_int_value();
    let val2 = cg.get_variable_value(value2)?.into_int_value();
    let res = match operation {
        BinOp::EQ => cg.builder.build_int_compare(inkwell::IntPredicate::EQ, val1, val2, "eq")?,
        BinOp::NEQ => cg.builder.build_int_compare(inkwell::IntPredicate::NE, val1, val2, "neq")?,
        BinOp::LESS => cg.builder.build_int_compare(inkwell::IntPredicate::SLT, val1, val2, "lt")?,
        BinOp::LESSEQ => cg.builder.build_int_compare(inkwell::IntPredicate::SLE, val1, val2, "lte")?,
        BinOp::GREATER => cg.builder.build_int_compare(inkwell::IntPredicate::SGT, val1, val2, "gt")?,
        BinOp::GREATEREQ => cg.builder.build_int_compare(inkwell::IntPredicate::SGE, val1, val2, "gte")?,
        _ => return Err(LLVMCodegenError::InvalidOperation),
    };
    return cg.create_variable(Type::Bool, res);
}

/// Compare two String with the given operation
pub fn compare_string_values<'ctx>(value1: &SmolVar<'ctx>, value2: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    todo!() // String not implemented yet
}

/// Compare two None with the given operation
pub fn compare_none_values<'ctx>(value1: &SmolVar<'ctx>, value2: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let res = match operation {
        BinOp::EQ => 1,
        BinOp::NEQ => 0,
        _ => return Err(LLVMCodegenError::InvalidOperation),
    };
    return cg.create_variable(Type::Bool, res);
}

/// Compare two Boolean with the given operation
pub fn compare_boolean_values<'ctx>(value1: &SmolVar<'ctx>, value2: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = cg.get_variable_value(value1)?.into_int_value();
    let val2 = cg.get_variable_value(value2)?.into_int_value();
    let res = match operation {
        BinOp::EQ => cg.builder.build_int_compare(IntPredicate::EQ, val1, val2, "bool_eq")?,
        BinOp::NEQ => cg.builder.build_int_compare(IntPredicate::NE, val1, val2, "bool_neq")?,
        BinOp::LESS => cg.builder.build_int_compare(IntPredicate::ULT, val1, val2, "bool_lt")?,
        BinOp::GREATER => cg.builder.build_int_compare(IntPredicate::UGT, val1, val2, "bool_gt")?,
        BinOp::LESSEQ => cg.builder.build_int_compare(IntPredicate::ULE, val1, val2, "bool_lte")?,
        BinOp::GREATEREQ => cg.builder.build_int_compare(IntPredicate::UGE, val1, val2, "bool_gte")?,
        _ => return Err(LLVMCodegenError::InvalidOperation),
    };
    return cg.create_variable(Type::Bool, res);
}

/// Compare two List with the given operation
pub fn compare_list_values<'ctx>(value1: &SmolVar<'ctx>, value2: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    todo!() // List not implemented yet
}

/// Compare two generic values with the given operation
pub fn compare_generic_values<'ctx>(value1: &SmolVar<'ctx>, value2: &SmolVar<'ctx>, operation: BinOp, cg: &CodeGen<'ctx>) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    todo!() // See how types are found in print.rs
}
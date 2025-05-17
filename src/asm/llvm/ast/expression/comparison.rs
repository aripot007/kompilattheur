use crate::{
    asm::{
        codegen::CodeGen,
        internal_global_constants::RuntimeErrorMsg,
        llvm::{assert_dyn_type, panic::smolpp_panic, smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::BinOp,
    typing::Type,
};
use inkwell::IntPredicate;
/// Compare two Integer with the given operation
pub fn compare_int_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = cg.get_variable_value(value1)?.into_int_value();
    let val2 = cg.get_variable_value(value2)?.into_int_value();
    let res = match operation {
        BinOp::EQ => cg
            .builder
            .build_int_compare(IntPredicate::EQ, val1, val2, "eq")?,
        BinOp::NEQ => cg
            .builder
            .build_int_compare(IntPredicate::NE, val1, val2, "neq")?,
        BinOp::LESS => cg
            .builder
            .build_int_compare(IntPredicate::SLT, val1, val2, "lt")?,
        BinOp::LESSEQ => cg
            .builder
            .build_int_compare(IntPredicate::SLE, val1, val2, "lte")?,
        BinOp::GREATER => cg
            .builder
            .build_int_compare(IntPredicate::SGT, val1, val2, "gt")?,
        BinOp::GREATEREQ => cg
            .builder
            .build_int_compare(IntPredicate::SGE, val1, val2, "gte")?,
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalid between two ints: {:?}",
                operation
            )))
        }
    };
    return cg.create_variable(Type::Bool, res);
}

/// Compare two String with the given operation
#[allow(unused)]
pub fn compare_string_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    smolpp_panic(cg, RuntimeErrorMsg::PanicNotImplemented, &[])?;
    return cg.create_variable(Type::Bool, cg.context.bool_type().const_int(0, false));
}

/// Compare two None with the given operation
pub fn compare_none_values<'ctx>(
    _value1: SmolVar<'ctx>,
    _value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let res = match operation {
        BinOp::EQ => cg.create_variable(Type::Bool, cg.context.bool_type().const_int(1, false)),
        BinOp::NEQ => cg.create_variable(Type::Bool, cg.context.bool_type().const_int(0, false)),
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalid between two None: {:?}",
                operation
            )))
        }
    };
    return res;
}

/// Compare two Boolean with the given operation
pub fn compare_boolean_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let val1 = cg.get_variable_value(value1)?.into_int_value();
    let val2 = cg.get_variable_value(value2)?.into_int_value();
    let res = match operation {
        BinOp::EQ => cg
            .builder
            .build_int_compare(IntPredicate::EQ, val1, val2, "bool_eq")?,
        BinOp::NEQ => cg
            .builder
            .build_int_compare(IntPredicate::NE, val1, val2, "bool_neq")?,
        BinOp::LESS => cg
            .builder
            .build_int_compare(IntPredicate::ULT, val1, val2, "bool_lt")?,
        BinOp::GREATER => cg
            .builder
            .build_int_compare(IntPredicate::UGT, val1, val2, "bool_gt")?,
        BinOp::LESSEQ => cg
            .builder
            .build_int_compare(IntPredicate::ULE, val1, val2, "bool_lte")?,
        BinOp::GREATEREQ => {
            cg.builder
                .build_int_compare(IntPredicate::UGE, val1, val2, "bool_gte")?
        }
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalide between booleans : {:?}",
                operation
            )))
        }
    };
    return cg.create_variable(Type::Bool, res);
}

/// Compare two List with the given operation
#[allow(unused)]
pub fn compare_list_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    smolpp_panic(cg, RuntimeErrorMsg::PanicNotImplemented, &[])?;
    return cg.create_variable(Type::Bool, cg.context.bool_type().const_int(0, false));
}
/// Compare two generic values with the given operation
/// Only EQ/NEQ are supported generically (with Bool↔Int assimilation).
/// Other operations produce a runtime type error.
pub fn compare_generic_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // Load runtime type tags
    let t1 = cg.get_variable_type(value1)?;
    let t2 = cg.get_variable_type(value2)?;

    // Prepare tags
    let bool_tag = cg
        .context
        .i8_type()
        .const_int(Type::Bool.get_bitmask() as u64, false);
    let int_tag = cg
        .context
        .i8_type()
        .const_int(Type::Int.get_bitmask() as u64, false);

    // Compare tags and assimilation cases
    let tag_eq = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t1, t2, "tag_eq")?;
    let is_b1 = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t1, bool_tag, "is_b1")?;
    let is_i2 = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t2, int_tag, "is_i2")?;
    let case1 = cg.builder.build_and(is_b1, is_i2, "b_i1")?;
    let is_i1 = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t1, int_tag, "is_i1")?;
    let is_b2 = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t2, bool_tag, "is_b2")?;
    let case2 = cg.builder.build_and(is_i1, is_b2, "b_i2")?;
    let assim = cg.builder.build_or(case1, case2, "assim")?;
    let dyn_eq = cg.builder.build_or(tag_eq, assim, "dyn_eq")?;

    let result_val = match operation {
        BinOp::EQ => dyn_eq,
        BinOp::NEQ => cg.builder.build_not(dyn_eq, "dyn_neq")?,
        _ => {
            smolpp_panic(cg, RuntimeErrorMsg::TypeError, &[])?;
            unreachable!()
        }
    };

    cg.create_variable(Type::Bool, result_val)
}

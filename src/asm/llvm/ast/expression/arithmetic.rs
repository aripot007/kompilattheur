use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{assert_type, smolvar::SmolVar, LLVMCodegenError},
    },
    typing::Type,
};

pub fn compute_mult<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
    return compute_mult_unchecked(x, y, cg);
}

pub fn compute_mult_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_mul(x_val, y_val, "mult")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_div<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
    return compute_div_unchecked(x, y, cg);
}

pub fn compute_div_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_signed_div(x_val, y_val, "div")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_mod<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
    return compute_mod_unchecked(x, y, cg);
}

pub fn compute_mod_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_signed_rem(x_val, y_val, "mod")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_sub<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
    return compute_sub_unchecked(x, y, cg);
}

pub fn compute_sub_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_sub(x_val, y_val, "sub")?;
    return cg.create_variable(Type::Int, res);
}

pub fn compute_add<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    assert_type(Type::Int, &x, cg, None)?;
    assert_type(Type::Int, &y, cg, None)?;
    return compute_add_unchecked(x, y, cg);
}

pub fn compute_add_unchecked<'ctx>(
    x: SmolVar<'ctx>,
    y: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let x_val = cg.get_variable_value(x)?.into_int_value();
    let y_val = cg.get_variable_value(y)?.into_int_value();
    let res = cg.builder.build_int_add(x_val, y_val, "add")?;
    return cg.create_variable(Type::Int, res);
}

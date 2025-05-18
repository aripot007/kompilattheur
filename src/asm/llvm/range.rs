use crate::{
    asm::{codegen::CodeGen, InternalFuctions},
    typing::Type,
};

use super::{assert_dyn_type, assert_type, LLVMCodegenError};

//Initialize the internal "range" function that will generate a list of numbers from 0 to n-1, given n
pub fn init_internal_range_function<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    let void_type = cg.context.void_type();
    let param_type = cg.smolpp_types.dynamic_type;

    // register function in module
    let func_type = void_type.fn_type(&[param_type.into()], false);
    let function = cg
        .module
        .add_function(InternalFuctions::Range.into(), func_type, None);

    // build function
    let entry = cg.context.append_basic_block(function, "range_entry");

    // switch builder to function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let param = function.get_first_param().unwrap().into_struct_value();

    // Assert that the type is an integer
    assert_type(
        Type::Int,
        &param,
        cg,
        Some("Expected i32 type for range parameter".to_string()),
    )?;

    // TODO: implement range()
    // returns a list of numbers from 0 to n-1 for given n

    // Return builder to main block
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}

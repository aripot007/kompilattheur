use crate::{
    asm::{codegen::CodeGen, InternalFuctions},
    typing::Type,
};

use super::{assert_type, LLVMCodegenError};

// Initialize the internal "list" function that will give a list for range(n) or a list
pub fn init_internal_list_function<'ctx>(cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    let void_type = cg.context.void_type();
    let param_type = cg.smolpp_types.dynamic_type;

    // register function in module
    let func_type = void_type.fn_type(&[param_type.into()], false);
    let function = cg
        .module
        .add_function(InternalFuctions::List.into(), func_type, None);

    // build function
    let entry = cg.context.append_basic_block(function, "list_entry");

    // switch builder to function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    let param = function.get_first_param().unwrap().into_struct_value();

    // Assert that the type is a list
    assert_type(
        Type::List,
        &param,
        cg,
        Some("Expected list type for list parameter".to_string()),
    )?;

    // TODO: implement list()
    // assert type before exec
    // list(range(n)) -> [0, 1, ... n-1],
    // list([4, 5]) -> [4, 5],
    // list(non list) -> error

    // Return builder to main block
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);

    return Ok(());
}

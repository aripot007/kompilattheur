use inkwell::AddressSpace;

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{memcpy::llvm_build_memcpy, smolvar::SmolVar},
        LLVMCodegenError,
    },
    typing::Type,
};

use super::smollist::LIST_STRUCT_LEN_INDEX;

/// Concatenate two lists
pub fn llvm_build_list_concat<'ctx>(
    list1: SmolVar<'ctx>,
    list2: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let list1_struct_ptr = cg.get_variable_value(list1)?.into_int_value();
    let list1_struct_ptr =
        cg.builder
            .build_int_to_ptr(list1_struct_ptr, ptr_type, "list1_struct_ptr")?;

    let list2_struct_ptr = cg.get_variable_value(list2)?.into_int_value();
    let list2_struct_ptr =
        cg.builder
            .build_int_to_ptr(list2_struct_ptr, ptr_type, "list2_struct_ptr")?;

    let list1_struct = cg
        .builder
        .build_load(cg.smolpp_types.list_type, list1_struct_ptr, "list1_struct")?
        .into_struct_value();
    let list2_struct = cg
        .builder
        .build_load(cg.smolpp_types.list_type, list2_struct_ptr, "list2_struct")?
        .into_struct_value();

    // Get needed capacity
    let list1_len = cg.build_get_list_length(list1_struct)?;
    let list2_len = cg.build_get_list_length(list2_struct)?;

    let res_len = cg.builder.build_int_add(list1_len, list2_len, "res_len")?;

    // Update res len and capacity

    let res_struct_ptr = cg.create_list_in_heap(res_len)?;

    let len_ptr = cg.builder.build_struct_gep(
        cg.smolpp_types.list_type,
        res_struct_ptr,
        LIST_STRUCT_LEN_INDEX,
        "res_len_ptr",
    )?;
    cg.builder.build_store(len_ptr, res_len)?;

    let res_array_ptr = cg.build_get_list_array_ptr_from_ptr(res_struct_ptr)?;
    let list1_array_ptr = cg.build_get_list_array_ptr(list1_struct)?;

    // Copy first list
    llvm_build_memcpy(
        list1_len,
        list1_array_ptr,
        res_array_ptr,
        cg.smolpp_types.dynamic_type,
        cg,
    )?;

    let shifted_res_array_ptr = unsafe {
        cg.builder.build_in_bounds_gep(
            cg.smolpp_types.dynamic_type,
            res_array_ptr,
            &[list1_len],
            "shifted_res_array_ptr",
        )
    }?;

    let list2_array_ptr = cg.build_get_list_array_ptr(list2_struct)?;

    llvm_build_memcpy(
        list2_len,
        list2_array_ptr,
        shifted_res_array_ptr,
        cg.smolpp_types.dynamic_type,
        cg,
    )?;

    let res_struct_ptr_int =
        cg.builder
            .build_ptr_to_int(res_struct_ptr, cg.context.i64_type(), "res_struct_ptr_int")?;

    return cg.create_variable(Type::List, res_struct_ptr_int);
}

use inkwell::AddressSpace;

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{memcpy::llvm_build_memcpy, smolvar::SmolVar},
        LLVMCodegenError,
    },
    typing::Type,
};

/// Concatenate two strs
pub fn llvm_build_string_concat<'ctx>(
    str1: SmolVar<'ctx>,
    str2: SmolVar<'ctx>,
    cg: &mut CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let str1_struct_ptr = cg.get_variable_value(str1)?.into_int_value();
    let str1_struct_ptr =
        cg.builder
            .build_int_to_ptr(str1_struct_ptr, ptr_type, "str1_struct_ptr")?;

    let str2_struct_ptr = cg.get_variable_value(str2)?.into_int_value();
    let str2_struct_ptr =
        cg.builder
            .build_int_to_ptr(str2_struct_ptr, ptr_type, "str2_struct_ptr")?;

    let str1_struct = cg
        .builder
        .build_load(cg.smolpp_types.string_type, str1_struct_ptr, "str1_struct")?
        .into_struct_value();

    let str2_struct = cg
        .builder
        .build_load(cg.smolpp_types.string_type, str2_struct_ptr, "str2_struct")?
        .into_struct_value();

    // Get needed capacity
    let str1_len = cg.build_get_string_length(str1_struct)?;
    let str2_len = cg.build_get_string_length(str2_struct)?;

    let res_len = cg.builder.build_int_add(str1_len, str2_len, "res_len")?;

    // Update res len and capacity

    let res_struct_ptr = cg.create_string_in_heap(res_len)?;

    let res_array_ptr = cg.build_get_string_array_ptr_from_ptr(res_struct_ptr)?;
    let str1_array_ptr = cg.build_get_string_array_ptr(str1_struct)?;

    // Copy first str
    llvm_build_memcpy(
        str1_len,
        str1_array_ptr,
        res_array_ptr,
        cg.context.i8_type(),
        cg,
    )?;

    let shifted_res_array_ptr = unsafe {
        cg.builder.build_in_bounds_gep(
            cg.context.i8_type(),
            res_array_ptr,
            &[str1_len],
            "shifted_res_array_ptr",
        )
    }?;

    let str2_array_ptr = cg.build_get_string_array_ptr(str2_struct)?;

    llvm_build_memcpy(
        str2_len,
        str2_array_ptr,
        shifted_res_array_ptr,
        cg.context.i8_type(),
        cg,
    )?;

    let res_struct_ptr_int =
        cg.builder
            .build_ptr_to_int(res_struct_ptr, cg.context.i64_type(), "res_struct_ptr_int")?;

    return cg.create_variable(Type::String, res_struct_ptr_int);
}

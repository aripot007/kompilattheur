use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{assert_type::assert_assignation_type, smolvar::SmolVar, LLVMCodegenError},
    },
    ast::nodes::Assign,
    common::diagnostic::Diagnostic,
};

use super::{compute_destination_ptr, llvm_compute_expr, MemoryPtr};

pub fn llvm_from_assign<'ctx>(
    assign: &Assign,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let expr_value: SmolVar<'ctx> = llvm_compute_expr(&assign.value, cg)?;

    let dest_ptr = compute_destination_ptr(&assign.destination, cg)?;

    // Store assignation result if destination pointer is storable
    match dest_ptr {
        MemoryPtr::Storable(dest_ptr) => {
            let dest_var =
                cg.builder
                    .build_load(cg.smolpp_types.dynamic_type, dest_ptr, "load_dest_value")?;

            assert_assignation_type(&dest_var.into_struct_value(), &expr_value, cg, Some(assign))?;

            // Store the result
            cg.builder.build_store(dest_ptr, expr_value)?;
        }
        MemoryPtr::ReadOnly(_) => {
            // Emit warning for discarded assignation
            cg.warnings
                .push(Diagnostic::discarded_assign_result(assign));
        }
    }

    return Ok(());
}

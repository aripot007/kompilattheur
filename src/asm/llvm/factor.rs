use inkwell::values::StructValue;

use crate::{asm::codegen::CodeGen, ast::nodes::Factor, common::diagnostic::Diagnostic, typing::Type};
use super::LLVMCodegenError;

pub fn llvm_compute_factor<'ctx>(factor: &Factor, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {

    match factor {
        Factor::String(file_element) => return llvm_compute_string_value(&file_element.element, cg),
        Factor::Integer(_)
        | Factor::True(_)
        | Factor::False(_)
        | Factor::None(_)
        | Factor::Identifier(_)
        | Factor::List(_)
        | Factor::Expr(_)
        | Factor::Call { identifier: _, args: _, localization: _ } => (),
    }

    cg.errors.push(Diagnostic::unimplemented_llvm(factor));
   
    return Err(());
}

fn llvm_compute_string_value<'ctx>(s: &String, cg: &mut CodeGen<'ctx>) -> Result<StructValue<'ctx>, LLVMCodegenError> {
    
    let type_discr = Type::String.get_discriminant();
    let type_discr_val = cg.context.i8_type().const_int(type_discr as u64, false);

    let str_const_ptr = cg.builder.build_global_string_ptr(&s, "string_const").unwrap();

    let val: StructValue<'ctx> = cg.smolpp_types.dynamic_type.const_named_struct(
        &[
            type_discr_val.into(),
            str_const_ptr.as_pointer_value().into(),
        ]
    );

    return Ok(val);

}

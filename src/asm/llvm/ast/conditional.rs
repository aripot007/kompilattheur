use inkwell::types::StructType;

use crate::asm::codegen::CodeGen;
use crate::asm::llvm::smolvar::SmolVar;
use crate::asm::llvm::{assert_type, LLVMCodegenError};
use crate::ast::nodes::Conditional;
use crate::common::diagnostic::Diagnostic;
use crate::typing::Type;

use super::llvm_compute_expr;

// 1 expr : la valeur
// 2 bloc : if et portentiellement else
// 3 blocs en LLVM : if (else) merge_res
// avec compute_expr : calcul valeur expr donne smolvar, vérifie que c'est un bool (assert_type)
// recupere val (get_val)
// on fait un int cmp avec 1 (True)
// on fait un branch if (et else)
// unconditinal branch vers le merge
// on remet le builder au bloc precedent

pub fn llvm_from_conditional<'ctx>(cond: &Conditional, cg: &mut CodeGen<'ctx>) -> Result<(), LLVMCodegenError> {
    // On fait en sorte d'être sûr d'avoir des booléens
    let expr_value: SmolVar<'ctx> = llvm_compute_expr(&cond.condition, cg)?;
    
    match cond.condition.expr_type {
        Some(Type::Bool) => {}
        _ => {
            assert_type(Type::Bool, &expr_value, cg, None)?;
        }
    }
    
    // Logique Branching
    //TODO : Romain
    return Ok(());
}

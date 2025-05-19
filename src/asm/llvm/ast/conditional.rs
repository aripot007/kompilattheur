use crate::asm::{
    codegen::CodeGen,
    get_internal_func,
    llvm::{assert_type, llvm_from_block, smolvar::SmolVar, LLVMCodegenError},
    InternalFuctions,
};
use crate::ast::nodes::Conditional;
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

pub fn llvm_from_conditional<'ctx>(
    cond: &Conditional,
    cg: &mut CodeGen<'ctx>,
) -> Result<(), LLVMCodegenError> {
    let expr_value: SmolVar<'ctx> = llvm_compute_expr(&cond.condition, cg)?;

    let current_function = cg.current_function;
    let then_block = cg.context.append_basic_block(current_function, "then");
    let else_block = cg.context.append_basic_block(current_function, "else");
    let merge_block = cg.context.append_basic_block(current_function, "merge");

    if cond.condition.expr_type != Some(Type::Bool) {
        //assert_type(Type::Bool, &expr_value, cg, None)?;
        // Cast e1_value to bool using the bool_cast_internal_function
        let call_boolean_llvm_value = cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::BoolCast),
            &[expr_value.into()],
            "not_bool_cast_call",
        )?;

        let boolean_llvm_value = call_boolean_llvm_value
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        
        cg.builder
            .build_conditional_branch(boolean_llvm_value, then_block, else_block)?;
    } else {
        let var_value = cg.get_variable_value(expr_value)?.into_int_value();
        let bool_value = cg
            .builder
            .build_int_cast(var_value, cg.context.bool_type(), "bool_if")?;

        cg.builder
            .build_conditional_branch(bool_value, then_block, else_block)?;
    }

    cg.builder.position_at_end(then_block);
    llvm_from_block(&cond.if_block, cg)?;
    cg.builder.build_unconditional_branch(merge_block)?;

    cg.builder.position_at_end(else_block);
    if let Some(else_block_node) = &cond.else_block {
        llvm_from_block(else_block_node, cg)?;
    }
    cg.builder.build_unconditional_branch(merge_block)?;

    cg.builder.position_at_end(merge_block);

    return Ok(());
}

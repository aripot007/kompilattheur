use crate::{
    asm::{
        codegen::CodeGen,
        get_internal_func, get_internal_global_const,
        internal_functions::InternalFuctions,
        internal_global_constants::RuntimeErrorMsg,
        llvm::{
            llvm_printf_custom, panic::smolpp_panic_with_unreachable, smolvar::SmolVar,
            LLVMCodegenError,
        },
        InternalGlobalConst,
    },
    ast::nodes::BinOp,
    typing::Type,
};
use inkwell::AddressSpace;
use inkwell::{
    values::{FunctionValue, IntValue},
    IntPredicate,
};

/// Compare two Integer with the given operation
pub fn compare_int_bool_range_values<'ctx>(
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
        _ => panic!("Invalid operation between two Int: {:?}", operation),
    };
    let res = cg
        .builder
        .build_int_cast(res, cg.context.i64_type(), "int_cast")?;
    return cg.create_variable(Type::Bool, res);
}

/// Compare two String with the given operation
pub fn compare_string_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let ptr_type = cg.context.ptr_type(AddressSpace::default());

    let str1_struct_ptr = cg.get_variable_value(value1)?.into_int_value();
    let str1_struct_ptr =
        cg.builder
            .build_int_to_ptr(str1_struct_ptr, ptr_type, "str1_struct_ptr")?;
    let str2_struct_ptr = cg.get_variable_value(value2)?.into_int_value();
    let str2_struct_ptr =
        cg.builder
            .build_int_to_ptr(str2_struct_ptr, ptr_type, "str2_struct_ptr")?;

    let str1_struct =
        cg.builder
            .build_load(cg.smolpp_types.string_type, str1_struct_ptr, "str1_stuct")?;
    let str2_struct =
        cg.builder
            .build_load(cg.smolpp_types.string_type, str2_struct_ptr, "str2_stuct")?;

    let cmp_val_callsite = cg.builder.build_call(
        get_internal_func!(cg, InternalFuctions::StrCmp),
        &[str1_struct.into(), str2_struct.into()],
        "cmp_val_callsite",
    )?;

    let cmp_val = cmp_val_callsite
        .try_as_basic_value()
        .unwrap_left()
        .into_int_value();

    let predicate = match operation {
        BinOp::LESS => IntPredicate::SLT,
        BinOp::LESSEQ => IntPredicate::SLE,
        BinOp::GREATER => IntPredicate::SGT,
        BinOp::GREATEREQ => IntPredicate::SGE,
        BinOp::EQ => IntPredicate::EQ,
        BinOp::NEQ => IntPredicate::NE,
        _ => panic!("No associated predicate for operation {}", operation),
    };

    let res = cg.builder.build_int_compare(
        predicate,
        cmp_val,
        cg.context.i64_type().const_zero(),
        "cmp_res",
    )?;
    let res = cg
        .builder
        .build_int_cast(res, cg.context.i64_type(), "cmp_res_int")?;

    return cg.create_variable(Type::Bool, res);
}

/// Compare two None with the given operation
pub fn compare_none_values<'ctx>(
    _value1: SmolVar<'ctx>,
    _value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    let res = match operation {
        BinOp::EQ => cg.create_variable(Type::Bool, cg.context.i64_type().const_int(1, false)),
        BinOp::NEQ => cg.create_variable(Type::Bool, cg.context.i64_type().const_int(0, false)),
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalid between two None: {:?}",
                operation
            )))
        }
    };
    return res;
}

/// Compare two List with the given operation
pub fn compare_list_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // Get the internal list_cmp function
    let list_cmp_func = cg
        .module
        .get_function(InternalFuctions::ListCmp.into())
        .unwrap();

    // Prepare arguments
    let val1_int = cg.get_variable_value(value1)?.into_int_value();
    let val2_int = cg.get_variable_value(value2)?.into_int_value();

    let ptr_type = cg.context.ptr_type(AddressSpace::default());
    let val1_ptr = cg
        .builder
        .build_int_to_ptr(val1_int, ptr_type, "list1_ptr")?;
    let val2_ptr = cg
        .builder
        .build_int_to_ptr(val2_int, ptr_type, "list2_ptr")?;

    let val1 = cg
        .builder
        .build_load(cg.smolpp_types.list_type, val1_ptr, "list1")?
        .into_struct_value();
    let val2 = cg
        .builder
        .build_load(cg.smolpp_types.list_type, val2_ptr, "list2")?
        .into_struct_value();

    // Call the list_cmp function to get the comparison result (-1, 0, 1)
    let args = &[val1.into(), val2.into()];
    let cmp_value = cg
        .builder
        .build_call(list_cmp_func, args, "list_cmp_call")?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    // Create comparison based on operation
    let result = match operation {
        BinOp::EQ => cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            cmp_value,
            cg.context.i8_type().const_zero(),
            "list_eq",
        )?,
        BinOp::NEQ => cg.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            cmp_value,
            cg.context.i8_type().const_zero(),
            "list_neq",
        )?,
        BinOp::LESS => cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            cmp_value,
            cg.context.i8_type().const_int(u64::MAX - 1, true), // -1 as i8
            "list_lt",
        )?,
        BinOp::GREATER => cg.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            cmp_value,
            cg.context.i8_type().const_int(1, false),
            "list_gt",
        )?,
        BinOp::LESSEQ => {
            let lt = cg.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                cmp_value,
                cg.context.i8_type().const_int(u64::MAX - 1, true), // -1 as i8
                "list_lt_part",
            )?;
            let eq = cg.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                cmp_value,
                cg.context.i8_type().const_zero(),
                "list_eq_part",
            )?;
            cg.builder.build_or(lt, eq, "list_lte")?
        }
        BinOp::GREATEREQ => {
            let gt = cg.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                cmp_value,
                cg.context.i8_type().const_int(1, false),
                "list_gt_part",
            )?;
            let eq = cg.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                cmp_value,
                cg.context.i8_type().const_zero(),
                "list_eq_part",
            )?;
            cg.builder.build_or(gt, eq, "list_gte")?
        }
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalid operation between two lists: {:?}",
                operation
            )))
        }
    };

    let result = cg
        .builder
        .build_int_cast(result, cg.context.i64_type(), "result_compare_list")?;

    return cg.create_variable(Type::Bool, result);
}

/// Compare two generic values with the given operation
/// Only EQ/NEQ are supported generically (with Bool↔Int assimilation).
/// Other operations produce a runtime type error.
///
/// EQ/NEQ ils sont apart, n'importe quel type, si type pareil EQ classique si type différent False pour EQ et True pour NEQ
/// List et String, si value1 est string on assert que value2 est string puis classique compare
/// si value1 est list on assert que value2 est list puis classique compare
/// Si tu as un bool ou un int tu assert que l'autre est un int ou un bool et tu fais la classique compare
pub fn compare_generic_values<'ctx>(
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    cg: &CodeGen<'ctx>,
) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
    // Call the generic compare function at runtime
    let call_site_value = match operation {
        BinOp::EQ => cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::GenericCompareEQ),
            &[value1.into(), value2.into()],
            "generic_compareEQ_call",
        )?,
        BinOp::NEQ => cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::GenericCompareNEQ),
            &[value1.into(), value2.into()],
            "generic_compareNEQ_call",
        )?,
        BinOp::LESS => cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::GenericCompareLESS),
            &[value1.into(), value2.into()],
            "generic_compareLESS_call",
        )?,
        BinOp::GREATER => cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::GenericCompareGREATER),
            &[value1.into(), value2.into()],
            "generic_compareGREATER_call",
        )?,
        BinOp::LESSEQ => cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::GenericCompareLESSEQ),
            &[value1.into(), value2.into()],
            "generic_compareLESSEQ_call",
        )?,
        BinOp::GREATEREQ => cg.builder.build_call(
            get_internal_func!(cg, InternalFuctions::GenericCompareGREATEREQ),
            &[value1.into(), value2.into()],
            "generic_compareGREATEREQ_call",
        )?,
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalide operation : {:?}",
                operation
            )))
        }
    };

    let return_value = call_site_value.try_as_basic_value().left().unwrap();

    Ok(return_value.into_struct_value())
}

pub fn init_internal_compare_generic_function<'ctx>(
    cg: &mut CodeGen<'ctx>,
    operation: BinOp,
) -> Result<(), LLVMCodegenError> {
    // Create the function
    let var_type = cg.smolpp_types.dynamic_type;

    // Register the function in the module
    let func_type = var_type.fn_type(&vec![var_type.into(); 2], false);

    let function = match operation {
        BinOp::EQ => {
            cg.module
                .add_function(InternalFuctions::GenericCompareEQ.into(), func_type, None)
        }
        BinOp::NEQ => {
            cg.module
                .add_function(InternalFuctions::GenericCompareNEQ.into(), func_type, None)
        }
        BinOp::LESS => {
            cg.module
                .add_function(InternalFuctions::GenericCompareLESS.into(), func_type, None)
        }
        BinOp::GREATER => cg.module.add_function(
            InternalFuctions::GenericCompareGREATER.into(),
            func_type,
            None,
        ),
        BinOp::LESSEQ => cg.module.add_function(
            InternalFuctions::GenericCompareLESSEQ.into(),
            func_type,
            None,
        ),
        BinOp::GREATEREQ => cg.module.add_function(
            InternalFuctions::GenericCompareGREATEREQ.into(),
            func_type,
            None,
        ),
        _ => {
            return Err(LLVMCodegenError::InvalidOperation(format!(
                "Invalide generation for this operation : {:?}",
                operation
            )))
        }
    };

    // Build the function
    let entry = cg
        .context
        .append_basic_block(function, "generic_compare_function_entry");

    // Switch builder to the function block
    cg.builder.position_at_end(entry);
    cg.current_function = function;

    // Get function parameter value
    let value1 = function
        .get_nth_param(0 as u32)
        .unwrap()
        .into_struct_value();
    let value2 = function
        .get_nth_param(1 as u32)
        .unwrap()
        .into_struct_value();

    // Load runtime type tags
    let t1 = cg.get_variable_type(value1)?;
    let t2 = cg.get_variable_type(value2)?;

    // Check dynamique à l'execution Si c'est le même type, on fait la compare classique
    let same_type = cg
        .builder
        .build_int_compare(IntPredicate::EQ, t1, t2, "dyn_eq")?;

    // Branch if the types are equal
    let parent_block = cg.builder.get_insert_block().unwrap();
    let then_block = cg
        .context
        .insert_basic_block_after(parent_block, "generic_compare_same_type");
    let else_block = cg
        .context
        .insert_basic_block_after(then_block, "generic_compare_different_type");
    let then_bool_or_int_block = cg
        .context
        .insert_basic_block_after(parent_block, "generic_compare_same_type");
    let else_bool_or_int_block = cg
        .context
        .insert_basic_block_after(then_block, "generic_compare_different_type");

    cg.builder
        .build_conditional_branch(same_type, then_block, else_block)?;

    // Case Same type
    cg.builder.position_at_end(then_block);

    build_switch_compare_generic_same_type(cg, function, value1, value2, operation, t1)?;

    // Case Different type
    cg.builder.position_at_end(else_block);

    // Create the comparaison calcul

    let i8_type = cg.context.i8_type();
    let bool_type = i8_type.const_int(Type::Bool.get_bitmask().into(), false);
    let int_type = i8_type.const_int(Type::Int.get_bitmask().into(), false);
    let mask = cg.builder.build_or(bool_type, int_type, "mask")?;
    let left_type = cg.builder.build_and(t1, mask, "left_type")?;
    let left_cond = cg.builder.build_int_compare(
        IntPredicate::NE,
        left_type,
        i8_type.const_zero(),
        "left_cond",
    )?;

    let right_type = cg.builder.build_and(t2, mask, "right_type")?;
    let right_cond = cg.builder.build_int_compare(
        IntPredicate::NE,
        right_type,
        i8_type.const_zero(),
        "right_cond",
    )?;

    let final_cond = cg.builder.build_and(left_cond, right_cond, "final_cond")?;

    // Si le type est Bool ou Int, alors on fait compare int
    cg.builder.build_conditional_branch(
        final_cond,
        then_bool_or_int_block,
        else_bool_or_int_block,
    )?;

    cg.builder.position_at_end(then_bool_or_int_block);
    let result = compare_int_bool_range_values(value1, value2, operation, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(else_bool_or_int_block);

    match operation {
        BinOp::EQ => {
            let res = cg.create_variable(Type::Bool, cg.context.i64_type().const_int(0, false))?;
            cg.builder.build_return(Some(&res))?;
        }
        BinOp::NEQ => {
            let res = cg.create_variable(Type::Bool, cg.context.i64_type().const_int(1, false))?;
            cg.builder.build_return(Some(&res))?;
        }
        _ => {
            smolpp_panic_with_unreachable(
                cg,
                RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
                &[t1.into()],
            )?;
        }
    }

    // Return builder to main block because it's init function
    cg.current_function = cg.main_function;
    cg.builder.position_at_end(cg.current_main_block);
    return Ok(());
}

fn build_switch_compare_generic_same_type<'ctx>(
    cg: &CodeGen<'ctx>,
    function: FunctionValue<'ctx>,
    value1: SmolVar<'ctx>,
    value2: SmolVar<'ctx>,
    operation: BinOp,
    t1: IntValue<'ctx>,
) -> Result<(), LLVMCodegenError> {
    // Contruire un switch case dynamique en fonction du type de value1
    // Create a switch based on the type field

    let case_none = cg
        .context
        .append_basic_block(function, "generic_compare_case_none");
    let case_int_bool_range = cg
        .context
        .append_basic_block(function, "generic_compare_case_int_bool_range");
    let case_string = cg
        .context
        .append_basic_block(function, "generic_compare_case_string");
    let case_list = cg
        .context
        .append_basic_block(function, "generic_compare_case_list");
    let default_block = cg
        .context
        .append_basic_block(function, "generic_compare_default");

    let i8_type = cg.context.i8_type();

    cg.builder.build_switch(
        t1,
        default_block,
        &[
            (
                i8_type.const_int(Type::None.get_bitmask().into(), false),
                case_none,
            ),
            (
                i8_type.const_int(Type::Bool.get_bitmask().into(), false),
                case_int_bool_range,
            ),
            (
                i8_type.const_int(Type::Int.get_bitmask().into(), false),
                case_int_bool_range,
            ),
            (
                i8_type.const_int(Type::String.get_bitmask().into(), false),
                case_string,
            ),
            (
                i8_type.const_int(Type::List.get_bitmask().into(), false),
                case_list,
            ),
            (
                i8_type.const_int(Type::Range.get_bitmask().into(), false),
                case_int_bool_range,
            ),
        ],
    )?;

    cg.builder.position_at_end(case_none);
    // Call the compare function for None only for EQ/NEQ

    match operation {
        BinOp::EQ | BinOp::NEQ => {
            let result = compare_none_values(value1, value2, operation, cg)?;
            cg.builder.build_return(Some(&result))?;
        }
        _ => {
            smolpp_panic_with_unreachable(
                cg,
                RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
                &[t1.into()],
            )?;
        }
    }

    cg.builder.position_at_end(case_int_bool_range);
    let result = compare_int_bool_range_values(value1, value2, operation, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(case_string);
    let result = compare_string_values(value1, value2, operation, cg)?;
    cg.builder.build_return(Some(&result))?;

    cg.builder.position_at_end(case_list);
    let result = compare_list_values(value1, value2, operation, cg)?;
    cg.builder.build_return(Some(&result))?;

    // Default case, print error message
    cg.builder.position_at_end(default_block);

    smolpp_panic_with_unreachable(
        cg,
        RuntimeErrorMsg::PanicInvalidInternalTypeCompareGeneric,
        &[t1.into()],
    )?;

    return Ok(());
}

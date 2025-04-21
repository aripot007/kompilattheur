use inkwell::values::{BasicValue, BasicValueEnum, IntValue, StructValue};

use crate::{asm::codegen::CodeGen, typing::Type};

use super::LLVMCodegenError;

pub(super) type SmolVar<'ctx> = StructValue<'ctx>;

//
// TODO: Replace StructValue with SmolVar in whole prog
//


impl<'ctx> CodeGen<'ctx> {
    
    /// Initialize the LLVM struct to represent dynamic types
    pub fn init_dynamic_type(&mut self) {
        let context = self.context;

        let var_type_discriminant = context.i8_type();

        let i64_type = context.i64_type();

        let ptr_size = self
            .target_machine
            .get_target_data()
            .get_pointer_byte_size(None)
            * 8;

        // Choose the largest type for the union
        let union_size = i64_type.get_bit_width().max(ptr_size);
        let var_value = context.custom_width_int_type(union_size);

        // Create the struct type
        self.smolpp_types.dynamic_type.set_body(
            &[
                var_type_discriminant.into(), // char type
                var_value.into(),             // Simulated union (either u64 or pointer)
            ],
            false,
        );
    }

    pub fn get_variable_type(&self, variable: SmolVar<'ctx>) -> Result<IntValue<'ctx>, LLVMCodegenError> {
        let val = self.builder.build_extract_value(variable, 0, "type_field")?.into_int_value();
        return Ok(val);
    }

    pub fn get_variable_value(&self, variable: SmolVar<'ctx>) -> Result<BasicValueEnum<'ctx>, LLVMCodegenError> {
        let val = self.builder.build_extract_value(variable, 1, "value_field")?;
        return Ok(val);
    }

    pub fn set_variable_type(&self, variable: SmolVar<'ctx>, t: Type) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
        return self.set_variable_type_bitmask(variable, t.get_bitmask());
    }

    pub fn set_variable_type_bitmask(&self, variable: SmolVar<'ctx>, bitmask: u8) -> Result<SmolVar<'ctx>, LLVMCodegenError> {
        let bitmask_val = self.context.i8_type().const_int(bitmask as u64, false);
        let res = self.builder.build_insert_value(variable, bitmask_val, 0, format!("set_type_{:#b}", bitmask).as_str())?;
        return Ok(res.into_struct_value());
    }

    pub fn set_variable_value<BV>(&self, variable: SmolVar<'ctx>, value: BV) -> Result<SmolVar<'ctx>, LLVMCodegenError> where BV: BasicValue<'ctx>, {
        let res = self.builder.build_insert_value(variable, value, 1, "set_value")?;
        return Ok(res.into_struct_value());
    }

    pub fn create_variable<BV>(&self, t: Type, value: BV) -> Result<SmolVar<'ctx>, LLVMCodegenError> where BV: BasicValue<'ctx> {
        let undef = self.smolpp_types.dynamic_type.get_undef();
        let var_type_discr_val = self.context.i8_type().const_int(t.get_bitmask() as u64, false);
        let with_type = self.builder.build_insert_value(undef, var_type_discr_val, 0, "with_type")?.into_struct_value();
        let full_struct = self.builder.build_insert_value(with_type, value.as_basic_value_enum(), 1, "with_value")?.into_struct_value();
        return Ok(full_struct);
    }
}

use crate::{
    asm::{
        codegen::CodeGen,
        llvm::{smolvar::SmolVar, LLVMCodegenError},
    },
    typing::Type,
};
use inkwell::{
    values::{IntValue, PointerValue, StructValue},
    AddressSpace,
};

pub type SmolString<'ctx> = StructValue<'ctx>;

impl<'ctx> CodeGen<'ctx> {
    /// Initialize the LLVM struct to represent strings
    ///
    /// strings are represented as follow in memory :
    /// ```
    /// struct string {
    ///     len: u64,  // Len of the string, excluding zero terminator
    ///     values: &char[len + 1],  // Pointer to the null terminated array of chars
    /// }
    /// ```
    pub fn init_string_type(&mut self) {
        let context = self.context;

        // Create the struct type
        self.smolpp_types.string_type.set_body(
            &[
                context.i64_type().into(),                        // Length
                context.ptr_type(AddressSpace::default()).into(), // Pointer to the array
            ],
            false,
        );
    }

    /// Create a string with the given capacity in the stack
    pub fn build_string(
        &self,
        capacity: IntValue<'ctx>,
    ) -> Result<SmolString<'ctx>, LLVMCodegenError> {
        let array_ptr =
            self.builder
                .build_array_alloca(self.context.i8_type(), capacity, "string_array")?;
        return self.build_string_struct(capacity, array_ptr);
    }

    /// Create a string in the heap and return the pointer to it
    pub fn create_string_in_heap(
        &self,
        capacity: IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        let array_ptr = self.builder.build_array_malloc(
            self.context.i8_type(),
            capacity,
            "string_heap_array",
        )?;
        let string_struct = self.build_string_struct(capacity, array_ptr)?;

        // Store the structure in the heap
        let string_ptr = self
            .builder
            .build_malloc(self.smolpp_types.string_type, "string")?;
        self.builder.build_store(string_ptr, string_struct)?;
        return Ok(string_ptr);
    }

    /// Create a string in the heap and return the pointer to it
    pub fn create_const_string_in_heap(
        &self,
        s: &String,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        let string_struct = self.build_const_string(s)?;

        // Store the structure in the heap
        let string_ptr = self
            .builder
            .build_malloc(self.smolpp_types.string_type, "string")?;
        self.builder.build_store(string_ptr, string_struct)?;
        return Ok(string_ptr);
    }

    /// Create a constant string
    pub fn build_const_string(&self, s: &String) -> Result<SmolString<'ctx>, LLVMCodegenError> {
        let str_const_ptr = self.builder.build_global_string_ptr(&s, "string_const")?;

        let len = self.context.i64_type().const_int(s.len() as u64, false);

        let string_struct = self.build_string_struct(len, str_const_ptr.as_pointer_value())?;

        return Ok(string_struct);
    }

    /// Create a string with the capable of storing len - 1 char
    pub fn build_string_struct(
        &self,
        len: IntValue<'ctx>,
        array_ptr: PointerValue<'ctx>,
    ) -> Result<SmolString<'ctx>, LLVMCodegenError> {
        let undef = self.smolpp_types.string_type.get_undef();
        let with_len = self
            .builder
            .build_insert_value(undef, len, 0, "with_len")?
            .into_struct_value();
        let full_struct = self
            .builder
            .build_insert_value(with_len, array_ptr, 1, "full_struct")?
            .into_struct_value();

        return Ok(full_struct);
    }

    /// Create a string variable capable of storing len - 1 char
    /// If `heap` is true, store the string data on the heap instead of the stack
    pub fn build_string_variable(
        &self,
        len: IntValue<'ctx>,
        heap: bool,
    ) -> Result<(SmolVar<'ctx>, PointerValue<'ctx>), LLVMCodegenError> {
        let string_ptr = match heap {
            true => self.create_string_in_heap(len)?,
            false => {
                // Create the string and store it on the stack
                let string_struct = self.build_string(len)?;

                let string_ptr = self
                    .builder
                    .build_alloca(self.smolpp_types.string_type, "string")?;
                self.builder.build_store(string_ptr, string_struct)?;
                string_ptr
            }
        };

        let val_type = self
            .smolpp_types
            .dynamic_type
            .get_field_type_at_index(1)
            .unwrap();
        let string_ptr_int =
            self.builder
                .build_ptr_to_int(string_ptr, val_type.into_int_type(), "string_ptr")?;

        return Ok((
            self.create_variable(Type::String, string_ptr_int)?,
            string_ptr,
        ));
    }

    /// Create a string variable capable of storing len - 1 char
    /// If `heap` is true, store the string data on the heap instead of the stack
    pub fn build_const_string_variable(
        &self,
        s: &String,
        heap: bool,
    ) -> Result<(SmolVar<'ctx>, PointerValue<'ctx>), LLVMCodegenError> {
        let string_ptr = match heap {
            true => self.create_const_string_in_heap(s)?,
            false => {
                // Create the string and store it on the stack
                let string_struct = self.build_const_string(s)?;

                let string_ptr = self
                    .builder
                    .build_alloca(self.smolpp_types.string_type, "string")?;
                self.builder.build_store(string_ptr, string_struct)?;
                string_ptr
            }
        };

        let val_type = self
            .smolpp_types
            .dynamic_type
            .get_field_type_at_index(1)
            .unwrap();
        let string_ptr_int =
            self.builder
                .build_ptr_to_int(string_ptr, val_type.into_int_type(), "string_ptr")?;

        return Ok((
            self.create_variable(Type::String, string_ptr_int)?,
            string_ptr,
        ));
    }

    /// Free a string variable stored in the heap
    /// This function MUST ONLY be used on string variables stored ON THE HEAP,
    /// ie created with `create_string_variable` with the `heap` parameter to `true`.
    pub fn build_free_string_variable(
        &self,
        string: SmolVar<'ctx>,
    ) -> Result<(), LLVMCodegenError> {
        let string_ptr = self.get_variable_value(string)?;
        return self.build_free_string(string_ptr.into_pointer_value());
    }

    /// Free a string structure stored in the heap
    /// This function MUST ONLY be used on string structures stored ON THE HEAP,
    /// ie created with the `create_string_in_heap` function.
    pub fn build_free_string(
        &self,
        string_struct_ptr: PointerValue<'ctx>,
    ) -> Result<(), LLVMCodegenError> {
        // Get the string struct from the heap
        let string_struct = self
            .builder
            .build_load(
                self.smolpp_types.string_type,
                string_struct_ptr,
                "string_struct",
            )?
            .into_struct_value();

        // Free the underlying array
        let array_ptr = self.build_get_string_array_ptr(string_struct)?;
        self.builder.build_free(array_ptr)?;

        // Free the string struct
        self.builder.build_free(string_struct_ptr)?;

        return Ok(());
    }

    pub fn build_get_string_length(
        &self,
        string: SmolString<'ctx>,
    ) -> Result<IntValue<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_extract_value(string, 0, "string_len")?
            .into_int_value());
    }

    fn build_set_string_length(
        &self,
        string: SmolString<'ctx>,
        len: IntValue<'ctx>,
    ) -> Result<SmolString<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_insert_value(string, len, 0, "set_string_len")?
            .into_struct_value());
    }

    pub fn build_get_string_array_ptr(
        &self,
        string: SmolString<'ctx>,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_extract_value(string, 1, "string_array_ptr")?
            .into_pointer_value());
    }

    pub fn build_get_string_array_ptr_from_ptr(
        &self,
        string_ptr: PointerValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        let ptr_ptr = self.builder.build_struct_gep(
            self.smolpp_types.string_type,
            string_ptr,
            1,
            "string_array_ptr",
        )?;
        let ptr = self.builder.build_load(
            self.context.ptr_type(AddressSpace::default()),
            ptr_ptr,
            "string_array_ptr",
        )?;
        return Ok(ptr.into_pointer_value());
    }

    fn build_set_string_array_ptr(
        &self,
        string: SmolString<'ctx>,
        ptr: PointerValue<'ctx>,
    ) -> Result<SmolString<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_insert_value(string, ptr, 1, "set_string_array_ptr")?
            .into_struct_value());
    }
}

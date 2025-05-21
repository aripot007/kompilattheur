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

pub(super) type SmolList<'ctx> = StructValue<'ctx>;

pub static LIST_STRUCT_LEN_INDEX: u32 = 0;
pub static LIST_STRUCT_CAPA_INDEX: u32 = 1;
pub static LIST_STRUCT_ARRAY_INDEX: u32 = 2;

macro_rules! printf_list_ptr {
    ($cg: expr, $list_ptr: expr) => {{
        use crate::asm::llvm::print::llvm_printf_custom;
        use crate::asm::InternalFuctions;
        let __list_struct = $cg
            .builder
            .build_load($cg.smolpp_types.list_type, $list_ptr, "__list_struct")?
            .into_struct_value();
        let __list_len = $cg.build_get_list_length(__list_struct)?;
        let __list_cap = $cg.build_get_list_capacity(__list_struct)?;
        let __list_ptr = $cg.build_get_list_array_ptr(__list_struct)?;
        llvm_printf_custom!(
            $cg,
            "List {len=%d, capa=%d, array=%p}",
            __list_len,
            __list_cap,
            __list_ptr
        );
    }};
}
pub(crate) use printf_list_ptr;

impl<'ctx> CodeGen<'ctx> {
    /// Initialize the LLVM struct to represent lists
    ///
    /// Lists are represented as follow in memory :
    /// ```
    /// struct list {
    ///     len: u64,  // Number of elements in the list
    ///     capacity: u64,  // Length of the underlying array
    ///     values: &SmolVar[capacity],  // Pointer to the array of given capacity
    /// }
    /// ```
    pub fn init_list_type(&mut self) {
        let context = self.context;

        // Create the struct type
        self.smolpp_types.list_type.set_body(
            &[
                context.i64_type().into(),                        // Length
                context.i64_type().into(),                        // Capacity
                context.ptr_type(AddressSpace::default()).into(), // Pointer to the array
            ],
            false,
        );
    }

    /// Create a list with the given capacity in the stack
    pub fn build_list(&self, capacity: IntValue<'ctx>) -> Result<SmolList<'ctx>, LLVMCodegenError> {
        let array_ptr = self.builder.build_array_alloca(
            self.smolpp_types.dynamic_type,
            capacity,
            "list_array",
        )?;
        return self.build_list_struct(capacity, array_ptr);
    }

    /// Create a list in the heap and return the pointer to it
    pub fn create_list_in_heap(
        &self,
        capacity: IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        let array_ptr = self.builder.build_array_malloc(
            self.smolpp_types.dynamic_type,
            capacity,
            "list_heap_array",
        )?;

        // Store the structure in the heap
        let list_ptr = self
            .builder
            .build_malloc(self.smolpp_types.list_type, "list")?;

        let struct_len_ptr = self.builder.build_struct_gep(
            self.smolpp_types.list_type,
            list_ptr,
            LIST_STRUCT_LEN_INDEX,
            "struct_len_ptr",
        )?;
        let struct_capa_ptr = self.builder.build_struct_gep(
            self.smolpp_types.list_type,
            list_ptr,
            LIST_STRUCT_CAPA_INDEX,
            "struct_capa_ptr",
        )?;
        let struct_array_ptr = self.builder.build_struct_gep(
            self.smolpp_types.list_type,
            list_ptr,
            LIST_STRUCT_ARRAY_INDEX,
            "struct_array_ptr",
        )?;

        self.builder
            .build_store(struct_len_ptr, self.context.i64_type().const_zero())?;
        self.builder.build_store(struct_capa_ptr, capacity)?;
        self.builder.build_store(struct_array_ptr, array_ptr)?;

        return Ok(list_ptr);
    }

    /// Create a list with the given capacity in the stack
    pub fn build_list_struct(
        &self,
        capacity: IntValue<'ctx>,
        array_ptr: PointerValue<'ctx>,
    ) -> Result<SmolList<'ctx>, LLVMCodegenError> {
        let undef = self.smolpp_types.list_type.get_undef();
        let len = self.context.i64_type().const_zero();
        let with_len = self
            .builder
            .build_insert_value(undef, len, LIST_STRUCT_LEN_INDEX, "with_len")?
            .into_struct_value();
        let with_capa = self
            .builder
            .build_insert_value(with_len, capacity, LIST_STRUCT_CAPA_INDEX, "with_capa")?
            .into_struct_value();
        let full_struct = self
            .builder
            .build_insert_value(with_capa, array_ptr, LIST_STRUCT_ARRAY_INDEX, "full_struct")?
            .into_struct_value();

        return Ok(full_struct);
    }

    /// Create a list variable
    /// If `heap` is true, store the list data on the heap instead of the stack
    pub fn build_list_variable(
        &self,
        capacity: IntValue<'ctx>,
        heap: bool,
    ) -> Result<(SmolVar<'ctx>, PointerValue<'ctx>), LLVMCodegenError> {
        let list_ptr = match heap {
            true => self.create_list_in_heap(capacity)?,
            false => {
                // Create the list and store it on the stack
                let list_struct = self.build_list(capacity)?;

                let list_ptr = self
                    .builder
                    .build_alloca(self.smolpp_types.list_type, "list")?;
                self.builder.build_store(list_ptr, list_struct)?;
                list_ptr
            }
        };

        let val_type = self
            .smolpp_types
            .dynamic_type
            .get_field_type_at_index(1)
            .unwrap();
        let list_ptr_int =
            self.builder
                .build_ptr_to_int(list_ptr, val_type.into_int_type(), "list_ptr")?;

        return Ok((self.create_variable(Type::List, list_ptr_int)?, list_ptr));
    }

    /// Free a list variable stored in the heap
    /// This function MUST ONLY be used on list variables stored ON THE HEAP,
    /// ie created with `create_list_variable` with the `heap` parameter to `true`.
    pub fn build_free_list_variable(&self, list: SmolVar<'ctx>) -> Result<(), LLVMCodegenError> {
        let list_ptr = self.get_variable_value(list)?;
        return self.build_free_list(list_ptr.into_pointer_value());
    }

    /// Free a list structure stored in the heap
    /// This function MUST ONLY be used on list structures stored ON THE HEAP,
    /// ie created with the `create_list_in_heap` function.
    pub fn build_free_list(
        &self,
        list_struct_ptr: PointerValue<'ctx>,
    ) -> Result<(), LLVMCodegenError> {
        // Get the list struct from the heap
        let list_struct = self
            .builder
            .build_load(self.smolpp_types.list_type, list_struct_ptr, "list_struct")?
            .into_struct_value();

        // Free the underlying array
        let array_ptr = self.build_get_list_array_ptr(list_struct)?;
        self.builder.build_free(array_ptr)?;

        // Free the list array
        self.builder.build_free(list_struct_ptr)?;

        return Ok(());
    }

    pub fn build_get_list_length(
        &self,
        list: SmolList<'ctx>,
    ) -> Result<IntValue<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_extract_value(list, LIST_STRUCT_LEN_INDEX, "list_len")?
            .into_int_value());
    }

    pub fn build_set_list_length(
        &self,
        list: SmolList<'ctx>,
        len: IntValue<'ctx>,
    ) -> Result<SmolList<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_insert_value(list, len, LIST_STRUCT_LEN_INDEX, "set_list_len")?
            .into_struct_value());
    }

    pub fn build_get_list_capacity(
        &self,
        list: SmolList<'ctx>,
    ) -> Result<IntValue<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_extract_value(list, LIST_STRUCT_CAPA_INDEX, "list_cap")?
            .into_int_value());
    }

    fn build_set_list_capacity(
        &self,
        list: SmolList<'ctx>,
        capacity: IntValue<'ctx>,
    ) -> Result<SmolList<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_insert_value(list, capacity, LIST_STRUCT_CAPA_INDEX, "set_list_cap")?
            .into_struct_value());
    }

    pub fn build_get_list_array_ptr(
        &self,
        list: SmolList<'ctx>,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_extract_value(list, LIST_STRUCT_ARRAY_INDEX, "list_array_ptr")?
            .into_pointer_value());
    }

    pub fn build_get_list_array_ptr_from_ptr(
        &self,
        list_ptr: PointerValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, LLVMCodegenError> {
        let ptr_ptr = self.builder.build_struct_gep(
            self.smolpp_types.list_type,
            list_ptr,
            LIST_STRUCT_ARRAY_INDEX,
            "list_array_ptr_ptr",
        )?;
        let ptr = self
            .builder
            .build_load(
                self.context.ptr_type(AddressSpace::default()),
                ptr_ptr,
                "list_array_ptr",
            )?
            .into_pointer_value();
        return Ok(ptr);
    }

    fn build_set_list_array_ptr(
        &self,
        list: SmolList<'ctx>,
        ptr: PointerValue<'ctx>,
    ) -> Result<SmolList<'ctx>, LLVMCodegenError> {
        return Ok(self
            .builder
            .build_insert_value(list, ptr, LIST_STRUCT_ARRAY_INDEX, "set_list_array_ptr")?
            .into_struct_value());
    }
}

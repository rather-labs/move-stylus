use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, InstrSeqId, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    data::{DATA_ABORT_MESSAGE_PTR_OFFSET, DATA_SLOT_DATA_PTR_OFFSET, RuntimeErrorData},
    error::RuntimeError,
    native_functions::NativeFunction,
    runtime::RuntimeFunction,
};

#[derive(Debug, thiserror::Error)]
pub enum WasmBuilderExtensionError {
    #[error("constant is too large to fit in u32")]
    ConstantTooLargeToFitInU32,

    #[error("constant is too large to fit in u64")]
    ConstantTooLargeToFitInU64,
}

pub trait WasmBuilderExtension {
    /// Negates the result of a boolean operation. User must be sure that the last value in the
    /// stack contains result of a boolean operation (0 or 1).
    ///
    /// If the last value in the stack is 0, after this operation will be 1
    /// If the last value in the stack is 1, after this operation will be 0
    fn negate(&mut self) -> &mut Self;

    /// Swaps the top two values of the stack.
    ///
    /// [..., v1, v2] --> swap() -> [..., v2, v1]
    ///
    /// The `LocalId` arguments are used as temp variables to perform the swap.
    fn swap(&mut self, v1: LocalId, v2: LocalId) -> &mut Self;

    /// Computes the address of an element in a vector.
    ///
    /// [..., ptr, index] --> vec_elem_ptr(size) -> [..., element_address]
    ///
    /// Where:
    /// - ptr: pointer to the vector
    /// - index: index of the element
    /// - size: size of each element in bytes
    fn vec_elem_ptr(&mut self, ptr: LocalId, index: LocalId, size: i32) -> &mut Self;

    /// Computes the address of an element in a vector.
    ///
    /// [..., ptr, index, size_local] --> vec_elem_ptr_dynamic() -> [..., element_address]
    ///
    /// Where:
    /// - ptr: pointer to the vector
    /// - index: index of the element
    /// - size_local: local variable containing the size of each element
    fn vec_elem_ptr_dynamic(
        &mut self,
        ptr: LocalId,
        index: LocalId,
        size_local: LocalId,
    ) -> &mut Self;

    /// Skips the length and capacity of a vector.
    ///
    /// [..., ptr] --> skip_vec_header() -> [..., ptr + 8]
    fn skip_vec_header(&mut self, ptr: LocalId) -> &mut Self;

    /// Adds the instructions to compute: DATA_SLOT_DATA_PTR_OFFSET + (32 - used_bytes_in_slot).
    fn slot_data_ptr_plus_offset(&mut self, used_bytes_in_slot: LocalId) -> &mut Self;

    /// Loads a i32 constant from a byte slice.
    ///
    /// The byte slice must be at most 4 bytes long. If it is shorter, it will be padded with zeros
    /// on the right.
    ///
    /// [..., ] --> load_i32_from_bytes() -> [..., i32_constant]
    fn load_i32_from_bytes(&mut self, bytes: &[u8])
    -> Result<&mut Self, WasmBuilderExtensionError>;

    /// Loads a i64 constant from a byte slice.
    ///
    /// The byte slice must be at most 8 bytes long. If it is shorter, it will be padded with zeros
    /// on the right.
    ///
    /// [..., ] --> load_i32_from_bytes() -> [..., i64_constant]
    fn load_i64_from_bytes(&mut self, bytes: &[u8])
    -> Result<&mut Self, WasmBuilderExtensionError>;

    /// Leaves in the stack the current position of the linear memory.
    fn get_memory_curret_position(&mut self, compilation_ctx: &CompilationContext) -> &mut Self;

    /// Calls a function and propagates the error in case the function aborts.
    fn call_native_function(
        &mut self,
        compilation_ctx: &CompilationContext,
        function_name: &str,
        module_id: &ModuleId,
        function_id: FunctionId,
    ) -> &mut Self;

    // Calls a runtime function and propagates the error in case the function aborts.
    fn call_runtime_function(
        &mut self,
        compilation_ctx: &CompilationContext,
        function_id: FunctionId,
        runtime_fn: &RuntimeFunction,
    ) -> &mut Self;

    // Call a runtime function conditionally returning or branching to `return_block_id`
    fn call_runtime_function_conditional_return(
        &mut self,
        compilation_ctx: &CompilationContext,
        function_id: FunctionId,
        runtime_fn: &RuntimeFunction,
        return_block_id: Option<InstrSeqId>,
    ) -> &mut Self;

    /// Adds the instructions to store the error message pointer at DATA_ABORT_MESSAGE_PTR_OFFSET and return 1 to indicate an error occurred.
    ///
    /// # Arguments
    /// * `module` - The WASM module
    /// * `compilation_ctx` - The compilation context
    /// * `return_type` - The return type of the function (`Some(ValType::I32)`, `Some(ValType::I64)`, or `None` for no return type)
    fn add_handle_error_instructions(
        &mut self,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        return_type: Option<ValType>,
    ) -> &mut Self;

    /// Adds the instructions to propagate the error by returning if the error message pointer at DATA_ABORT_MESSAGE_PTR_OFFSET is not null.
    fn add_propagate_error_instructions(
        &mut self,
        compilation_ctx: &CompilationContext,
        return_block_id: Option<InstrSeqId>,
    ) -> &mut Self;

    /// Adds the instructions to handle a specific runtime error.
    ///
    /// # Arguments
    /// * `module` - The WASM module
    /// * `compilation_ctx` - The compilation context
    /// * `return_type` - The return type of the function (`Some(ValType::I32)`, `Some(ValType::I64)`, or `None` for no return type)
    /// * `runtime_error_data` - Runtime error data
    /// * `runtime_error` - The runtime error to return
    fn return_error(
        &mut self,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        return_type: Option<ValType>,
        runtime_error_data: &mut RuntimeErrorData,
        runtime_error: RuntimeError,
    ) -> &mut Self;
}

impl WasmBuilderExtension for InstrSeqBuilder<'_> {
    fn negate(&mut self) -> &mut Self {
        // 1 != 1 = 0
        // 1 != 0 = 1
        self.i32_const(1).binop(BinaryOp::I32Ne)
    }

    fn swap(&mut self, v1: LocalId, v2: LocalId) -> &mut Self {
        self.local_set(v1).local_set(v2).local_get(v1).local_get(v2)
    }

    fn vec_elem_ptr(&mut self, ptr: LocalId, index: LocalId, size: i32) -> &mut Self {
        self.skip_vec_header(ptr)
            .local_get(index)
            .i32_const(size)
            .binop(BinaryOp::I32Mul)
            .binop(BinaryOp::I32Add)
    }

    fn vec_elem_ptr_dynamic(
        &mut self,
        ptr: LocalId,
        index: LocalId,
        size_local: LocalId,
    ) -> &mut Self {
        self.skip_vec_header(ptr)
            .local_get(index)
            .local_get(size_local)
            .binop(BinaryOp::I32Mul)
            .binop(BinaryOp::I32Add)
    }

    fn skip_vec_header(&mut self, ptr: LocalId) -> &mut Self {
        self.local_get(ptr).i32_const(8).binop(BinaryOp::I32Add)
    }

    fn slot_data_ptr_plus_offset(&mut self, slot_offset: LocalId) -> &mut Self {
        // Check if 0 < offset <= 32
        self.local_get(slot_offset).unop(UnaryOp::I32Eqz).if_else(
            None,
            |then| {
                then.unreachable();
            },
            |else_| {
                else_
                    .local_get(slot_offset)
                    .i32_const(32)
                    .binop(BinaryOp::I32GtU)
                    .if_else(
                        None,
                        |then| {
                            then.unreachable();
                        },
                        |_| {},
                    );
            },
        );

        self.i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(32)
            .local_get(slot_offset)
            .binop(BinaryOp::I32Sub)
            .binop(BinaryOp::I32Add)
    }

    fn load_i32_from_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<&mut Self, WasmBuilderExtensionError> {
        if bytes.len() > 4 {
            return Err(WasmBuilderExtensionError::ConstantTooLargeToFitInU32);
        }

        // pad to 4 bytes on the right
        let mut num_bytes = [0u8; 4];
        num_bytes[..bytes.len()].copy_from_slice(bytes);

        self.i32_const(i32::from_le_bytes(num_bytes));

        Ok(self)
    }

    fn load_i64_from_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<&mut Self, WasmBuilderExtensionError> {
        if bytes.len() > 8 {
            return Err(WasmBuilderExtensionError::ConstantTooLargeToFitInU64);
        }

        // pad to 8 bytes on the right
        let mut num_bytes = [0u8; 8];
        num_bytes[..bytes.len()].copy_from_slice(bytes);

        self.i64_const(i64::from_le_bytes(num_bytes));

        Ok(self)
    }

    fn get_memory_curret_position(&mut self, compilation_ctx: &CompilationContext) -> &mut Self {
        self.i32_const(0).call(compilation_ctx.allocator)
    }

    fn call_native_function(
        &mut self,
        compilation_ctx: &CompilationContext,
        function_name: &str,
        module_id: &ModuleId,
        function_id: FunctionId,
    ) -> &mut Self {
        // Call the function
        self.call(function_id);

        // If the function may result in a runtime error, we need to handle it
        if NativeFunction::can_abort(function_name, module_id) {
            self.add_propagate_error_instructions(compilation_ctx, None);
        }

        self
    }

    fn call_runtime_function(
        &mut self,
        compilation_ctx: &CompilationContext,
        function_id: FunctionId,
        runtime_fn: &RuntimeFunction,
    ) -> &mut Self {
        self.call(function_id);

        // If the function may result in a runtime error, we need to handle it
        if runtime_fn.can_abort() {
            self.add_propagate_error_instructions(compilation_ctx, None);
        }

        self
    }

    fn call_runtime_function_conditional_return(
        &mut self,
        compilation_ctx: &CompilationContext,
        function_id: FunctionId,
        runtime_fn: &RuntimeFunction,
        return_block_id: Option<InstrSeqId>,
    ) -> &mut Self {
        self.call(function_id);

        // If the function may result in a runtime error, we need to handle it
        if runtime_fn.can_abort() {
            // If the function aborts, propagate the error
            self.add_propagate_error_instructions(compilation_ctx, return_block_id);
        }

        self
    }

    fn add_propagate_error_instructions(
        &mut self,
        compilation_ctx: &CompilationContext,
        return_block_id: Option<InstrSeqId>,
    ) -> &mut Self {
        // If the function aborts, propagate the error
        self.block(None, |b| {
            let block_id = b.id();
            b.i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            b.i32_const(0xBADF00D);

            // TODO: if calling from a runtime fn, this could be an u64
            if let Some(return_block_id) = return_block_id {
                b.br(return_block_id);
            } else {
                b.return_();
            }
        });

        self
    }

    fn add_handle_error_instructions(
        &mut self,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        return_type: Option<ValType>,
    ) -> &mut Self {
        let encoded_error_ptr = module.locals.add(ValType::I32);
        self.local_set(encoded_error_ptr);

        // Store the ptr at DATA_ABORT_MESSAGE_PTR_OFFSET
        self.i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
            .local_get(encoded_error_ptr)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Return 0xBADF00D to indicate an error occurred
        // This value serves only to maintain stack balance.
        // It will be ignored if the caller returns nothing.
        // Depending on the return type, we push an i32, i64 or nothing to the stack.
        match return_type {
            Some(ValType::I32) => {
                self.i32_const(0xBADF00D).return_();
            }
            Some(ValType::I64) => {
                self.i64_const(0xBADF00D).return_();
            }
            Some(_) | None => {
                self.return_();
            }
        }

        self
    }

    fn return_error(
        &mut self,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        return_type: Option<ValType>,
        runtime_error_data: &mut RuntimeErrorData,
        runtime_error: RuntimeError,
    ) -> &mut Self {
        self.i32_const(runtime_error_data.get(module, compilation_ctx.memory_id, runtime_error));

        self.add_handle_error_instructions(module, compilation_ctx, return_type);
        self
    }
}

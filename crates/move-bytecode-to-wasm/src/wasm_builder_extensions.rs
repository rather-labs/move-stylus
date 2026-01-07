use walrus::{
    InstrSeqBuilder, LocalId,
    ir::{BinaryOp, UnaryOp},
};

use crate::{CompilationContext, data::DATA_SLOT_DATA_PTR_OFFSET};

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
}

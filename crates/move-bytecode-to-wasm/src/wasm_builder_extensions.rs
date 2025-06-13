use walrus::{InstrSeqBuilder, LocalId, ir::BinaryOp};

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
}

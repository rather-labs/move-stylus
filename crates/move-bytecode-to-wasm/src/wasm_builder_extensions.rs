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
}

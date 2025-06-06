use walrus::{InstrSeqBuilder, ir::BinaryOp};

pub trait WasmBuilderExtension {
    /// This operation negates the result of a boolean operation. User must be sure that the last
    /// value in the stack contains result of a boolean operation (0 or 1).
    ///
    /// If the last value in the stack is 0, after this operation will be 1
    /// If the last value in the stack is 1, after this operation will be 0
    fn negate(&mut self) -> &mut Self;
}

impl WasmBuilderExtension for InstrSeqBuilder<'_> {
    fn negate(&mut self) -> &mut Self {
        // 1 != 1 = 0
        // 1 != 0 = 1
        self.i32_const(1).binop(BinaryOp::I32Ne)
    }
}

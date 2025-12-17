use walrus::{InstrSeqBuilder, LocalId, ir::BinaryOp};

use crate::{abi_types::error::AbiError, vm_handled_types::bytes::Bytes};

impl Bytes {
    pub fn add_unpack_instructions(
        builder: &mut InstrSeqBuilder,
        reader_pointer: LocalId,
    ) -> Result<(), AbiError> {
        // Push the reader pointer to the stack.
        // It already points to the bytes we need to unpack.
        builder.local_get(reader_pointer);

        // Advance the reader pointer by 32
        // Bytes<M> is not a dynamic type, so the M bytes are left aligned and the rest are padded with 0s til 32
        builder
            .local_get(reader_pointer)
            .i32_const(32_i32)
            .binop(BinaryOp::I32Add)
            .local_set(reader_pointer);

        Ok(())
    }
}

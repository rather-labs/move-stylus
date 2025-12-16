use walrus::{InstrSeqBuilder, LocalId, ir::BinaryOp};

use crate::{abi_types::error::AbiError, vm_handled_types::bytes::Bytes4};

impl Bytes4 {
    pub fn add_unpack_instructions(
        builder: &mut InstrSeqBuilder,
        reader_pointer: LocalId,
    ) -> Result<(), AbiError> {
        // Push the reader pointer to the stack.
        // It already points to the 4 bytes we need to unpack.
        builder.local_get(reader_pointer);

        // Advance the reader pointer by 32
        // bytes<M> is not a dynamic type, so the first M bytes are left aligned and the rest are padded with 0s til 32
        builder
            .local_get(reader_pointer)
            .i32_const(32_i32)
            .binop(BinaryOp::I32Add)
            .local_set(reader_pointer);

        Ok(())
    }
}

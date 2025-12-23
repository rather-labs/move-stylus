use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::CompilationContext;
use crate::runtime::RuntimeFunction;
use crate::{abi_types::error::AbiError, vm_handled_types::bytes::Bytes};

impl Bytes {
    // Adds instructions to pack fixed bytes (Bytes<M>) into memory
    pub fn add_pack_instructions(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        bytes_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        builder.local_get(bytes_pointer);

        // Allocate 32 bytes for the fixed bytes data
        let data_pointer = module.locals.add(ValType::I32);
        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_tee(data_pointer);

        // Copy the bytes data to the allocated memory
        builder
            .local_get(bytes_pointer)
            .i32_const(32)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

        // Push the data pointer to the stack
        builder.local_get(data_pointer);

        Ok(())
    }

    // Adds instructions to pack dynamic bytes (Bytes) into memory
    pub fn add_pack_instructions_dynamic(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        bytes_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        Ok(())
    }
}

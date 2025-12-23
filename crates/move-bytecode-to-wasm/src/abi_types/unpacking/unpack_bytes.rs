use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::CompilationContext;
use crate::runtime::RuntimeFunction;
use crate::{
    abi_types::error::AbiError, translation::intermediate_types::IntermediateType,
    vm_handled_types::bytes::Bytes,
};

impl Bytes {
    pub fn add_unpack_instructions(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        itype: &IntermediateType,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        if Bytes::is_dynamic(itype, compilation_ctx)? {
            // Dynamic bytes (e.g., Bytes) use ABI encoding:
            // - 32 bytes: offset to data
            // - 32 bytes: length of data
            // - n bytes: actual data bytes
            //
            // Fixed bytes (e.g., Bytes4) are just M bytes (left-aligned) padded with zeros up to 32 bytes.

            // Setup runtime functions
            let validate_pointer_fn =
                RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx))?;
            let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

            // Extract data offset: read offset from reader_pointer (bytes 28-31) and add to calldata base
            builder.local_get(reader_pointer).call(validate_pointer_fn);
            let data_reader_pointer = module.locals.add(ValType::I32);
            builder
                .local_get(reader_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 28,
                    },
                )
                .call(swap_i32_bytes_function)
                .local_get(calldata_reader_pointer)
                .binop(BinaryOp::I32Add)
                .local_set(data_reader_pointer);

            // Advance reader pointer to next argument
            builder
                .local_get(reader_pointer)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .local_set(reader_pointer);

            // Load bytes length
            builder
                .local_get(data_reader_pointer)
                .call(validate_pointer_fn);
            let len = module.locals.add(ValType::I32);
            builder
                .local_get(data_reader_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 28,
                    },
                )
                .call(swap_i32_bytes_function)
                .local_set(len);

            // Allocate 8 bytes
            // Store the dynamic bytes length at offset 0
            // Store the pointer to the dynamic bytes data at offset 4
            let bytes_data_pointer = module.locals.add(ValType::I32);
            builder
                .i32_const(8)
                .call(compilation_ctx.allocator)
                .local_set(bytes_data_pointer);

            // Store length at offset 0
            builder.local_get(bytes_data_pointer).local_get(len).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Store data pointer at offset 4 (points to actual bytes data from the calldata, 32 bytes past length header)
            builder
                .local_get(bytes_data_pointer)
                .local_get(data_reader_pointer)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 4,
                    },
                );

            // Return bytes_data_pointer on stack
            builder.local_get(bytes_data_pointer);
        } else {
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
        }
        Ok(())
    }
}

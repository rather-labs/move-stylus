use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    runtime::RuntimeFunction, translation::intermediate_types::vector::IVector,
    vm_handled_types::string::String_,
};

use crate::CompilationContext;

use super::error::AbiUnpackError;

impl String_ {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiUnpackError> {
        // Big-endian to Little-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

        let data_reader_pointer = module.locals.add(ValType::I32);

        // The ABI encoded value of a dynamic type is a reference to the location of the
        // values in the call data.
        // We are just assuming that the max value can fit in 32 bits, otherwise we cannot reference WASM memory
        // If the value is greater than 32 bits, the WASM program will panic
        for i in 0..7 {
            block.block(None, |inner_block| {
                let inner_block_id = inner_block.id();

                inner_block
                    .local_get(reader_pointer)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            // Abi encoded value is Big endian
                            offset: i * 4,
                        },
                    )
                    .i32_const(0)
                    .binop(BinaryOp::I32Eq)
                    .br_if(inner_block_id);

                inner_block.unreachable();
            });
        }

        block
            .local_get(reader_pointer)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    // Abi encoded value is Big endian
                    offset: 28,
                },
            )
            .call(swap_i32_bytes_function)
            .local_get(calldata_reader_pointer)
            .binop(BinaryOp::I32Add)
            .local_set(data_reader_pointer); // This references the vector actual data

        // The reader will only be incremented until the next argument
        block
            .local_get(reader_pointer)
            .i32_const(32) // The size of the argument we just read
            .binop(BinaryOp::I32Add)
            .local_set(reader_pointer);

        // First 256 bits of the vector are the length
        // We are handling the length as u32 so the first 28 bytes are not needed
        // We need to ensure that they are zero to avoid runtime errors
        for i in 0..7 {
            block.block(None, |inner_block| {
                let inner_block_id = inner_block.id();

                inner_block
                    .local_get(data_reader_pointer)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            // Abi encoded value is Big endian
                            offset: i * 4,
                        },
                    )
                    .i32_const(0)
                    .binop(BinaryOp::I32Eq)
                    .br_if(inner_block_id);

                inner_block.unreachable();
            });
        }

        // Vector length: current number of elements in the vector
        let length = module.locals.add(ValType::I32);

        block
            .local_get(data_reader_pointer)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    // Abi encoded value is Big endian
                    offset: 28,
                },
            )
            .call(swap_i32_bytes_function)
            .local_set(length);

        // increment data reader pointer
        block
            .local_get(data_reader_pointer)
            .i32_const(32)
            .binop(BinaryOp::I32Add)
            .local_set(data_reader_pointer);

        let vector_pointer = module.locals.add(ValType::I32);
        let writer_pointer = module.locals.add(ValType::I32);

        IVector::allocate_vector_with_header(
            block,
            compilation_ctx,
            vector_pointer,
            length,
            length,
            4,
        );
        block.local_get(vector_pointer).local_set(writer_pointer);

        // increment pointer
        block
            .local_get(writer_pointer)
            .i32_const(8) // The size of the length + capacity written above
            .binop(BinaryOp::I32Add)
            .local_set(writer_pointer);

        // Copy elements
        let i = module.locals.add(ValType::I32);
        block.i32_const(0).local_set(i);

        let calldata_reader_pointer = module.locals.add(ValType::I32);
        block
            .local_get(data_reader_pointer)
            .local_set(calldata_reader_pointer);

        block.loop_(None, |loop_block| {
            let loop_block_id = loop_block.id();

            loop_block.local_get(writer_pointer);

            loop_block.local_get(data_reader_pointer).load(
                compilation_ctx.memory_id,
                LoadKind::I32_8 {
                    kind: ExtendedLoad::ZeroExtend,
                },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            loop_block.store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // increment reader pointer
            loop_block
                .local_get(data_reader_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(data_reader_pointer);

            // increment writer pointer
            loop_block
                .local_get(writer_pointer)
                .i32_const(4)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);

            // increment i
            loop_block
                .local_get(i)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_tee(i);

            loop_block
                .local_get(length)
                .binop(BinaryOp::I32LtU)
                .br_if(loop_block_id);
        });

        let struct_ptr = module.locals.add(ValType::I32);
        // Create the struct pointing to the vector
        block
            .i32_const(4)
            .call(compilation_ctx.allocator)
            .local_tee(struct_ptr);

        // Save the vector pointer as the first value
        block.local_get(vector_pointer).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Return the String struct
        block.local_get(struct_ptr);

        Ok(())
    }
}

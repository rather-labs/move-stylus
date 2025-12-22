use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    abi_types::error::AbiError, runtime::RuntimeFunction,
    translation::intermediate_types::vector::IVector, vm_handled_types::string::String_,
    wasm_builder_extensions::WasmBuilderExtension,
};

use crate::CompilationContext;

impl String_ {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        // Big-endian to Little-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;
        // Validate that the pointer fits in 32 bits
        let validate_pointer_fn =
            RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx))?;

        let data_reader_pointer = module.locals.add(ValType::I32);

        // The ABI encoded value of a dynamic type is a reference to the location of the
        // values in the call data.
        block.local_get(reader_pointer).call(validate_pointer_fn);

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

        // Validate that the data reader pointer fits in 32 bits
        block
            .local_get(data_reader_pointer)
            .call(validate_pointer_fn);

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

        // Increment data reader pointer
        block
            .local_get(data_reader_pointer)
            .i32_const(32)
            .binop(BinaryOp::I32Add)
            .local_set(data_reader_pointer);

        let vector_pointer = module.locals.add(ValType::I32);
        let writer_pointer = module.locals.add(ValType::I32);

        // Allocate space for the vector
        // Each u8 element takes 1 byte
        IVector::allocate_vector_with_header(
            block,
            compilation_ctx,
            vector_pointer,
            length,
            length,
            1,
        );
        block.local_get(vector_pointer).local_set(writer_pointer);

        // Set writer pointer to the start of the vector data
        block
            .skip_vec_header(writer_pointer)
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

            // Increment data reader pointer by 1 byte to point to the next u8 element
            loop_block
                .local_get(data_reader_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(data_reader_pointer);

            // Increment writer pointer by 1 byte to point to the next u8 element
            loop_block
                .local_get(writer_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);

            // Increment i
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

use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

pub fn unpack_string_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;
    // Validate that the pointer fits in 32 bits
    let validate_pointer_fn =
        RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx), None)?;

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::UnpackString.name().to_owned())
        .func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    let data_reader_pointer = module.locals.add(ValType::I32);

    // The ABI encoded value of a dynamic type is a reference to the location of the
    // values in the call data.
    builder.local_get(reader_pointer).call(validate_pointer_fn);

    builder
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

    // Advance the reader pointer by 32
    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    // Validate that the data reader pointer fits in 32 bits
    builder
        .local_get(data_reader_pointer)
        .call(validate_pointer_fn);

    // Vector length: current number of elements in the vector
    let length = module.locals.add(ValType::I32);

    builder
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
    builder
        .local_get(data_reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_set(data_reader_pointer);

    let vector_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Allocate space for the vector
    // Each u8 element takes 1 byte
    let allocate_vector_with_header_function =
        RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx), None)?;
    builder
        .local_get(length)
        .local_get(length)
        .i32_const(1)
        .call(allocate_vector_with_header_function)
        .local_set(vector_pointer);

    builder.local_get(vector_pointer).local_set(writer_pointer);

    // Set writer pointer to the start of the vector data
    builder
        .skip_vec_header(writer_pointer)
        .local_set(writer_pointer);

    // Copy elements
    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);

    builder.loop_(None, |loop_block| {
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
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(struct_ptr);

    // Save the vector pointer as the first value
    builder.local_get(vector_pointer).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Return the String struct
    builder.local_get(struct_ptr);

    Ok(function.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}

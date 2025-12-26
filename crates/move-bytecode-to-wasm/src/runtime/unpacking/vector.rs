use crate::{
    CompilationContext,
    abi_types::{error::AbiError, unpacking::Unpackable},
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::{IntermediateType, vector::IVector},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

/// Generates a runtime function that unpacks a vector from ABI-encoded calldata.
///
/// This function:
/// 1. Reads the pointer to the vector data from calldata
/// 2. Reads the vector length
/// 3. Allocates memory for the vector
/// 4. Unpacks each element recursively
/// 5. Returns a pointer to the unpacked vector
///
/// # WASM Function Arguments
/// * `reader_pointer` - (i32): pointer to the current position in the ABI-encoded data
/// * `calldata_base_pointer` - (i32): pointer to the start of the calldata
///
/// # WASM Function Returns
/// * `vector_pointer` - (i32): pointer to the unpacked vector in memory
pub fn unpack_vector_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::UnpackVector.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_base_pointer = module.locals.add(ValType::I32);

    // Runtime functions
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;
    let validate_pointer_fn =
        RuntimeFunction::ValidatePointer32Bit.get(module, Some(compilation_ctx))?;

    let data_reader_pointer = module.locals.add(ValType::I32);

    // The ABI encoded value of a dynamic type is a reference to the location of the
    // values in the call data.

    // Validate that the pointer fits in 32 bits
    builder.local_get(reader_pointer).call(validate_pointer_fn);

    // Load the pointer to the data, swap it to little-endian and add that to the calldata reader pointer.
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
        .local_get(calldata_base_pointer)
        .binop(BinaryOp::I32Add)
        .local_set(data_reader_pointer); // This references the vector actual data

    // Increment the reader pointer to next argument
    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

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

    let data_size = inner
        .wasm_memory_data_size()
        .map_err(RuntimeFunctionError::from)?;
    IVector::allocate_vector_with_header(
        &mut builder,
        compilation_ctx,
        vector_pointer,
        length,
        length,
        data_size,
    );

    // Set the writer pointer to the start of the vector data
    builder
        .skip_vec_header(vector_pointer)
        .local_set(writer_pointer);

    // Copy elements
    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);

    let calldata_base_pointer_ = module.locals.add(ValType::I32);
    builder
        .local_get(data_reader_pointer)
        .local_set(calldata_base_pointer_);

    let mut inner_result: Result<(), AbiError> = Ok(());
    builder.loop_(None, |loop_block| {
        inner_result = (|| {
            let loop_block_id = loop_block.id();

            loop_block.local_get(writer_pointer);
            // This will leave in the stack [pointer/value i32/i64, length i32]
            inner.add_unpack_instructions(
                loop_block,
                module,
                data_reader_pointer,
                calldata_base_pointer_,
                compilation_ctx,
            )?;

            // Store the value
            loop_block.store(
                compilation_ctx.memory_id,
                inner.store_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // increment writer pointer
            loop_block.local_get(writer_pointer);
            loop_block.i32_const(data_size);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_set(writer_pointer);

            // increment i
            loop_block.local_get(i);
            loop_block.i32_const(1);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_tee(i);

            loop_block.local_get(length);
            loop_block.binop(BinaryOp::I32LtU);
            loop_block.br_if(loop_block_id);

            Ok(())
        })();
    });

    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    builder.local_get(vector_pointer);

    // Check for errors from the loop
    inner_result?;

    Ok(function.finish(
        vec![reader_pointer, calldata_base_pointer],
        &mut module.funcs,
    ))
}

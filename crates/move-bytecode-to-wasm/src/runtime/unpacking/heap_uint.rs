use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn unpack_u128_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i128_bytes_function =
        RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx))?;
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<128>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    // The data is padded 16 bytes to the right
    let unpacked_pointer = module.locals.add(ValType::I32);
    function_body
        .local_get(reader_pointer)
        .i32_const(16)
        .binop(BinaryOp::I32Add);
    function_body
        .i32_const(16)
        .call(compilation_ctx.allocator)
        .local_tee(unpacked_pointer)
        .call(swap_i128_bytes_function);

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(unpacked_pointer);

    function_builder.name(RuntimeFunction::UnpackU128.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_u256_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i256_bytes_function =
        RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx))?;
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<256>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    function_body.local_get(reader_pointer);
    let unpacked_pointer = module.locals.add(ValType::I32);
    function_body
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(unpacked_pointer)
        .call(swap_i256_bytes_function);

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(unpacked_pointer);

    function_builder.name(RuntimeFunction::UnpackU256.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_address_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Address::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    let unpacked_pointer = module.locals.add(ValType::I32);
    function_body
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(unpacked_pointer);

    for i in 0..4 {
        function_body
            .local_get(unpacked_pointer)
            .local_get(reader_pointer)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            )
            .store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
    }

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(unpacked_pointer);

    function_builder.name(RuntimeFunction::UnpackAddress.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

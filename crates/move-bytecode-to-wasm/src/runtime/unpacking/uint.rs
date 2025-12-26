use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

pub fn unpack_u32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size = module.locals.add(ValType::I32);

    // Load the value
    function_body
        .local_get(reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        )
        .call(swap_i32_bytes_function);

    // Set the global reader pointer to reader pointer + encoded size
    function_body
        .local_get(reader_pointer)
        .local_get(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_builder.name(RuntimeFunction::UnpackU32.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer, encoded_size], &mut module.funcs))
}

pub fn unpack_u64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None)?;

    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<64>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)?;

    // Load the value
    function_body
        .local_get(reader_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 24,
            },
        )
        .call(swap_i64_bytes_function);

    // Increment reader pointer
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size as i32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_builder.name(RuntimeFunction::UnpackU64.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}

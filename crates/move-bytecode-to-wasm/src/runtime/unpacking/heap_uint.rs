use crate::{
    CompilationContext,
    abi_types::error::AbiError,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use alloy_sol_types::{SolType, sol_data};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

pub fn unpack_u128_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i128_bytes_function =
        RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx), None)?;
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::UnpackU128.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<128>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    // The data is padded 16 bytes to the right
    let unpacked_pointer = module.locals.add(ValType::I32);
    builder
        .local_get(reader_pointer)
        .i32_const(16)
        .binop(BinaryOp::I32Add);
    builder
        .i32_const(16)
        .call(compilation_ctx.allocator)
        .local_tee(unpacked_pointer)
        .call(swap_i128_bytes_function);

    // Increment reader pointer
    builder
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(unpacked_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_u256_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i256_bytes_function =
        RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx), None)?;
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::UnpackU256.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<256>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    builder.local_get(reader_pointer);
    let unpacked_pointer = module.locals.add(ValType::I32);
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(unpacked_pointer)
        .call(swap_i256_bytes_function);

    // Increment reader pointer
    builder
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(unpacked_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

pub fn unpack_address_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::UnpackAddress.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Address::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    let unpacked_pointer = module.locals.add(ValType::I32);
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(unpacked_pointer);

    builder
        .local_get(unpacked_pointer)
        .local_get(reader_pointer)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Increment reader pointer
    builder
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(unpacked_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::unpacking::Unpackable,
        data::RuntimeErrorData,
        test_compilation_context,
        test_tools::{INITIAL_MEMORY_OFFSET, build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    /// Test helper for unpacking heap-allocated types (u128, u256, address)
    fn unpack_heap_uint(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, ctx_globals) =
            build_module(Some(data.len() as i32));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_set(args_pointer);

        int_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                None,
                args_pointer,
                args_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, INITIAL_MEMORY_OFFSET + data.len() as i32);

        let global_next_free_memory_pointer = global_next_free_memory_pointer
            .get(&mut store)
            .i32()
            .unwrap();
        assert_eq!(
            global_next_free_memory_pointer,
            (INITIAL_MEMORY_OFFSET as usize + expected_result_bytes.len() + data.len()) as i32
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    #[test]
    fn test_unpack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IU128;

        let data = SolType::abi_encode_params(&(88,));
        unpack_heap_uint(&data, int_type.clone(), &88u128.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_heap_uint(&data, int_type.clone(), &(IntType::MAX - 1).to_le_bytes());
    }

    #[test]
    fn test_unpack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IU256;

        let data = SolType::abi_encode_params(&(U256::from(88),));
        unpack_heap_uint(&data, int_type.clone(), &U256::from(88).to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_heap_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        unpack_heap_uint(
            &data,
            int_type.clone(),
            &(IntType::MAX - U256::from(1)).to_le_bytes::<32>(),
        );
    }

    #[test]
    fn test_unpack_address() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::IAddress;

        let data = SolType::abi_encode_params(&(Address::ZERO,));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0x1234567890abcdef1234567890abcdef12345678"),));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_heap_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE"),));
        unpack_heap_uint(&data, int_type.clone(), &data);
    }
}

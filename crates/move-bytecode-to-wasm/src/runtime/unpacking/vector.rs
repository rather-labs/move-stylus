use crate::{
    CompilationContext,
    abi_types::{error::AbiError, unpacking::Unpackable},
    data::RuntimeErrorData,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, UnaryOp},
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
    runtime_error_data: &mut RuntimeErrorData,
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
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;
    let validate_pointer_fn = RuntimeFunction::ValidatePointer32Bit.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;

    let data_reader_pointer = module.locals.add(ValType::I32);

    // The ABI encoded value of a dynamic type is a reference to the location of the
    // values in the call data.

    // Validate that the pointer fits in 32 bits
    builder.local_get(reader_pointer).call_runtime_function(
        compilation_ctx,
        validate_pointer_fn,
        &RuntimeFunction::ValidatePointer32Bit,
        Some(ValType::I32),
    );

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
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    // Validate that the data reader pointer fits in 32 bits
    builder
        .local_get(data_reader_pointer)
        .call_runtime_function(
            compilation_ctx,
            validate_pointer_fn,
            &RuntimeFunction::ValidatePointer32Bit,
            Some(ValType::I32),
        );

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

    // Allocate space for the vector
    let allocate_vector_with_header_function =
        RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx), None)?;
    builder
        .local_get(length)
        .local_get(length)
        .i32_const(data_size)
        .call(allocate_vector_with_header_function)
        .local_set(vector_pointer);

    // Set the writer pointer to the start of the vector data
    builder
        .skip_vec_header(vector_pointer)
        .local_set(writer_pointer);

    let mut inner_result: Result<(), AbiError> = Ok(());
    builder.block(None, |block| {
        let block_id = block.id();
        block
            .local_get(length)
            .unop(UnaryOp::I32Eqz)
            .br_if(block_id);

        // Copy elements
        let i = module.locals.add(ValType::I32);
        block.i32_const(0).local_set(i);

        let calldata_base_pointer_ = module.locals.add(ValType::I32);
        block
            .local_get(data_reader_pointer)
            .local_set(calldata_base_pointer_);

        block.loop_(None, |loop_block| {
            inner_result = (|| {
                let loop_block_id = loop_block.id();

                loop_block.local_get(writer_pointer);
                // This will leave in the stack [pointer/value i32/i64, length i32]
                inner.add_unpack_instructions(
                    None,
                    loop_block,
                    module,
                    None,
                    Some(ValType::I32),
                    data_reader_pointer,
                    calldata_base_pointer_,
                    compilation_ctx,
                    Some(runtime_error_data),
                    None,
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
    });

    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(vector_pointer);

    // Check for errors from the loop
    inner_result?;

    Ok(function.finish(
        vec![reader_pointer, calldata_base_pointer],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        abi_types::unpacking::Unpackable,
        data::RuntimeErrorData,
        test_compilation_context,
        test_tools::{INITIAL_MEMORY_OFFSET, build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::{SolType, sol};
    use std::{fs::write, sync::Arc};
    use walrus::{FunctionBuilder, ValType};

    /// Test helper for unpacking vector types
    fn unpack_vec(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, ctx_globals) =
            build_module(Some(data.len() as i32));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        int_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    // ============================================================================
    // Vector Types - Simple Element Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_u8_empty() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params::<(Vec<u8>,)>(&(vec![],));
        let expected_result_bytes =
            [0u32.to_le_bytes().as_slice(), 0u32.to_le_bytes().as_slice()].concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u8() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u16() {
        type SolType = sol!((uint16[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(vec![1, 2],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u32() {
        type SolType = sol!((uint32[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u64() {
        type SolType = sol!((uint64[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Vector Types - Heap-Allocated Element Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_u128() {
        type SolType = sol!((uint128[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 36) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 52) as u32)
                .to_le_bytes()
                .as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u256() {
        type SolType = sol!((uint256[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IU256));

        let data =
            SolType::abi_encode_params(&(vec![U256::from(1), U256::from(2), U256::from(3)],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 52) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 84) as u32)
                .to_le_bytes()
                .as_slice(),
            U256::from(1).to_le_bytes::<32>().as_slice(),
            U256::from(2).to_le_bytes::<32>().as_slice(),
            U256::from(3).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_address() {
        type SolType = sol!((address[],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IAddress));

        let data = SolType::abi_encode_params(&(vec![
            address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            address!("0xcccccccccccccccccccccccccccccccccccccccc"),
        ],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 52) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 84) as u32)
                .to_le_bytes()
                .as_slice(),
            &[0; 12],
            address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").as_slice(),
            &[0; 12],
            address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").as_slice(),
            &[0; 12],
            address!("0xcccccccccccccccccccccccccccccccccccccccc").as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Nested Vector Types
    // ============================================================================

    #[test]
    fn test_unpack_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU32,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 16) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 36) as u32)
                .to_le_bytes()
                .as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let int_type = IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(), // len
            2u32.to_le_bytes().as_slice(), // capacity
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 16) as u32)
                .to_le_bytes()
                .as_slice(), // first element pointer
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 84) as u32)
                .to_le_bytes()
                .as_slice(), // second element pointer
            3u32.to_le_bytes().as_slice(), // first element length
            3u32.to_le_bytes().as_slice(), // first element capacity
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 36) as u32)
                .to_le_bytes()
                .as_slice(), // first element - first value pointer
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 52) as u32)
                .to_le_bytes()
                .as_slice(), // first element - second value pointer
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 68) as u32)
                .to_le_bytes()
                .as_slice(), // first element - third value pointer
            1u128.to_le_bytes().as_slice(), // first element - first value
            2u128.to_le_bytes().as_slice(), // first element - second value
            3u128.to_le_bytes().as_slice(), // first element - third value
            3u32.to_le_bytes().as_slice(), // second element length
            3u32.to_le_bytes().as_slice(), // second element capacity
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 104) as u32)
                .to_le_bytes()
                .as_slice(), // second element - first value pointer
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 120) as u32)
                .to_le_bytes()
                .as_slice(), // second element - second value pointer
            ((INITIAL_MEMORY_OFFSET + data.len() as i32 + 136) as u32)
                .to_le_bytes()
                .as_slice(), // second element - third value pointer
            4u128.to_le_bytes().as_slice(), // second element - first value
            5u128.to_le_bytes().as_slice(), // second element - second value
            6u128.to_le_bytes().as_slice(), // second element - third value
        ]
        .concat();
        unpack_vec(&data, int_type, &expected_result_bytes);
    }

    // ============================================================================
    // Fuzz Tests
    // ============================================================================

    #[test]
    fn test_unpack_vector_u8_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU8));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u8>>()
            .for_each(|values: &Vec<u8>| {
                // Explicitly encode as uint8[] array type, not bytes
                type SolType = sol!((uint8[],));
                let abi_data = SolType::abi_encode_params(&(values.clone(),));

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                // Verify length and capacity
                let len = values.len() as u32;
                let mut result_memory_data = vec![0; 8 + values.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    &result_memory_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &result_memory_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );
                assert_eq!(&result_memory_data[8..], values.as_slice(), "Data mismatch");

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_u16_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU16));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u16>>()
            .for_each(|values: &Vec<u16>| {
                // Explicitly encode as uint16[] array type
                type SolType = sol!((uint16[],));
                let abi_data = SolType::abi_encode_params(&(values.clone(),));

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = values.len() as u32;
                let mut result_memory_data = vec![0; 8 + values.len() * 2];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    &result_memory_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &result_memory_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );

                for (i, value) in values.iter().enumerate() {
                    let offset = 8 + i * 2;
                    assert_eq!(
                        &result_memory_data[offset..offset + 2],
                        value.to_le_bytes().as_slice(),
                        "Data mismatch at index {}",
                        i
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_u32_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU32));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u32>>()
            .for_each(|values: &Vec<u32>| {
                // Explicitly encode as uint32[] array type
                type SolType = sol!((uint32[],));
                let abi_data = SolType::abi_encode_params(&(values.clone(),));

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = values.len() as u32;
                let mut result_memory_data = vec![0; 8 + values.len() * 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    &result_memory_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &result_memory_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );

                for (i, value) in values.iter().enumerate() {
                    let offset = 8 + i * 4;
                    assert_eq!(
                        &result_memory_data[offset..offset + 4],
                        value.to_le_bytes().as_slice(),
                        "Data mismatch at index {}",
                        i
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_u64_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU64));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u64>>()
            .for_each(|values: &Vec<u64>| {
                // Explicitly encode as uint64[] array type
                type SolType = sol!((uint64[],));
                let abi_data = SolType::abi_encode_params(&(values.clone(),));

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = values.len() as u32;
                let mut result_memory_data = vec![0; 8 + values.len() * 8];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    &result_memory_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &result_memory_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );

                for (i, value) in values.iter().enumerate() {
                    let offset = 8 + i * 8;
                    assert_eq!(
                        &result_memory_data[offset..offset + 8],
                        value.to_le_bytes().as_slice(),
                        "Data mismatch at index {}",
                        i
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_u128_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU128));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(10000));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<u128>>()
            .for_each(|values: &Vec<u128>| {
                // Explicitly encode as uint128[] array type
                type SolType = sol!((uint128[],));
                let abi_data = SolType::abi_encode_params(&(values.clone(),));

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = values.len() as u32;
                let mut header_data = vec![0; 8 + values.len() * 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut header_data,
                    )
                    .unwrap();

                assert_eq!(
                    &header_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &header_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );

                // For u128, values are heap-allocated, so we need to read pointers
                for (i, &value) in values.iter().enumerate() {
                    let ptr_offset = 8 + i * 4;
                    let value_ptr = u32::from_le_bytes(
                        header_data[ptr_offset..ptr_offset + 4].try_into().unwrap(),
                    );

                    let mut value_data = [0; 16];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            value_ptr as usize,
                            &mut value_data,
                        )
                        .unwrap();

                    assert_eq!(
                        u128::from_le_bytes(value_data),
                        value,
                        "Value mismatch at index {}",
                        i
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_u256_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IU256));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(10000));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<[u8; 32]>>()
            .for_each(|values: &Vec<[u8; 32]>| {
                let abi_data = values
                    .iter()
                    .map(|v| U256::from_le_bytes(*v))
                    .collect::<Vec<U256>>()
                    .abi_encode();

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = values.len() as u32;
                let mut header_data = vec![0; 8 + len as usize * 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut header_data,
                    )
                    .unwrap();

                assert_eq!(
                    &header_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &header_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );

                // For u256, values are heap-allocated, so we need to read pointers
                for (i, value) in values.iter().enumerate() {
                    let ptr_offset = 8 + i * 4;
                    let value_ptr = u32::from_le_bytes(
                        header_data[ptr_offset..ptr_offset + 4].try_into().unwrap(),
                    );

                    let mut value_data = vec![0; 32];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            value_ptr as usize,
                            &mut value_data,
                        )
                        .unwrap();

                    assert_eq!(
                        U256::from_le_slice(&value_data),
                        U256::from_le_bytes(*value),
                        "Value mismatch at index {}",
                        i
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_address_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IAddress));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(10000));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<[u8; 20]>>()
            .for_each(|values: &Vec<[u8; 20]>| {
                let addresses: Vec<Address> = values.iter().map(|v| Address::from(*v)).collect();
                let abi_data = addresses.abi_encode();

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = values.len() as u32;
                let mut header_data = vec![0; 8 + len as usize * 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut header_data,
                    )
                    .unwrap();

                assert_eq!(
                    &header_data[0..4],
                    len.to_le_bytes().as_slice(),
                    "Length mismatch"
                );
                assert_eq!(
                    &header_data[4..8],
                    len.to_le_bytes().as_slice(),
                    "Capacity mismatch"
                );

                // For address, values are heap-allocated, so we need to read pointers
                for (i, value) in values.iter().enumerate() {
                    let ptr_offset = 8 + i * 4;
                    let value_ptr = u32::from_le_bytes(
                        header_data[ptr_offset..ptr_offset + 4].try_into().unwrap(),
                    );

                    let mut value_data = vec![0; 32];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            value_ptr as usize,
                            &mut value_data,
                        )
                        .unwrap();

                    // Address is stored as 12 zero bytes + 20 address bytes
                    assert_eq!(
                        &value_data[0..12],
                        &[0u8; 12],
                        "Address padding mismatch at index {}",
                        i
                    );
                    assert_eq!(
                        &value_data[12..32],
                        value,
                        "Address mismatch at index {}",
                        i
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_vector_u32_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU32,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(10000));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<Vec<u32>>>()
            .for_each(|values: &Vec<Vec<u32>>| {
                let abi_data = values.abi_encode();

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                println!("Result ptr {result_ptr}");

                let outer_len = values.len() as u32;
                let mut outer_header = vec![0; 8 + outer_len as usize * 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut outer_header,
                    )
                    .unwrap();

                assert_eq!(
                    &outer_header[0..4],
                    outer_len.to_le_bytes().as_slice(),
                    "Outer length mismatch"
                );

                // Read each inner vector
                for (i, inner_vec) in values.iter().enumerate() {
                    let ptr_offset = 8 + i * 4;
                    let inner_ptr = u32::from_le_bytes(
                        outer_header[ptr_offset..ptr_offset + 4].try_into().unwrap(),
                    );

                    let inner_len = inner_vec.len() as u32;
                    let mut inner_data = vec![0; 8 + inner_len as usize * 4];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            inner_ptr as usize,
                            &mut inner_data,
                        )
                        .unwrap();

                    assert_eq!(
                        &inner_data[0..4],
                        inner_len.to_le_bytes().as_slice(),
                        "Inner length mismatch at index {}",
                        i
                    );

                    for (j, &value) in inner_vec.iter().enumerate() {
                        let offset = 8 + j * 4;
                        assert_eq!(
                            u32::from_le_bytes(inner_data[offset..offset + 4].try_into().unwrap()),
                            value,
                            "Inner value mismatch at [{}, {}]",
                            i,
                            j
                        );
                    }
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_vector_vector_u128_fuzz() {
        use alloy_sol_types::SolValue;
        use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

        let vector_type = IntermediateType::IVector(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(10000));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        let func_body_id = func_body.id();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .local_tee(args_pointer)
            .local_set(calldata_reader_pointer);

        vector_type
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                Some(func_body_id),
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = crate::test_tools::get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", Some(linker));

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<Vec<Vec<u128>>>()
            .for_each(|values: &Vec<Vec<u128>>| {
                let abi_data = values.abi_encode();

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let outer_len = values.len() as u32;
                let mut outer_header = vec![0; 8 + outer_len as usize * 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut outer_header,
                    )
                    .unwrap();

                assert_eq!(
                    &outer_header[0..4],
                    outer_len.to_le_bytes().as_slice(),
                    "Outer length mismatch"
                );

                // Read each inner vector
                for (i, inner_vec) in values.iter().enumerate() {
                    let ptr_offset = 8 + i * 4;
                    let inner_ptr = u32::from_le_bytes(
                        outer_header[ptr_offset..ptr_offset + 4].try_into().unwrap(),
                    );

                    let inner_len = inner_vec.len() as u32;
                    let mut inner_header = vec![0; 8 + inner_len as usize * 4];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            inner_ptr as usize,
                            &mut inner_header,
                        )
                        .unwrap();

                    assert_eq!(
                        &inner_header[0..4],
                        inner_len.to_le_bytes().as_slice(),
                        "Inner length mismatch at index {}",
                        i
                    );

                    // For u128, values are heap-allocated
                    for (j, &value) in inner_vec.iter().enumerate() {
                        let value_ptr_offset = 8 + j * 4;
                        let value_ptr = u32::from_le_bytes(
                            inner_header[value_ptr_offset..value_ptr_offset + 4]
                                .try_into()
                                .unwrap(),
                        );

                        let mut value_data = vec![0; 16];
                        memory
                            .read(
                                &mut *store.0.borrow_mut(),
                                value_ptr as usize,
                                &mut value_data,
                            )
                            .unwrap();

                        assert_eq!(
                            u128::from_le_bytes(value_data.try_into().unwrap()),
                            value,
                            "Inner value mismatch at [{}, {}]",
                            i,
                            j
                        );
                    }
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

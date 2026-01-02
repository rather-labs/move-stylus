use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

pub fn pack_u128_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Little-endian to Big-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::PackU128.name().to_owned())
        .func_body();

    let value_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Pack 2 i64 values, loading from right to left, storing left to right
    for i in 0..2 {
        // Get writer pointer on stack
        builder.local_get(writer_pointer);

        // Load from value pointer (right to left)
        builder.local_get(value_pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 8 - i * 8,
            },
        );

        // Little-endian to Big-endian
        builder.call(swap_i64_bytes_function);

        // Store at writer pointer (left to right, left-padded to 32 bytes)
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                // ABI is left-padded to 32 bytes
                offset: 16 + i * 8,
            },
        );
    }

    Ok(function.finish(vec![value_pointer, writer_pointer], &mut module.funcs))
}

pub fn pack_u256_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Little-endian to Big-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::PackU256.name().to_owned())
        .func_body();

    let value_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Pack 4 i64 values, loading from right to left, storing left to right
    for i in 0..4 {
        // Get writer pointer on stack
        builder.local_get(writer_pointer);

        // Load from value pointer (right to left)
        builder.local_get(value_pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 24 - i * 8,
            },
        );

        // Little-endian to Big-endian
        builder.call(swap_i64_bytes_function);

        // Store at writer pointer (left to right)
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: i * 8,
            },
        );
    }

    Ok(function.finish(vec![value_pointer, writer_pointer], &mut module.funcs))
}

pub fn pack_address_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::PackAddress.name().to_owned())
        .func_body();

    let value_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Address is packed as a u160, but endianness is not relevant
    builder
        .local_get(writer_pointer)
        .local_get(value_pointer)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    Ok(function.finish(vec![value_pointer, writer_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::SolValue;
    use rstest::rstest;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::packing::Packable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    #[rstest]
    #[case(
        IntermediateType::IU128,
        &128128128128u128.to_le_bytes(),
        &128128128128_u128.abi_encode()
    )]
    #[case(
        IntermediateType::IU128,
        &u128::MAX.to_le_bytes(),
        &u128::MAX.abi_encode()
    )]
    #[case(
        IntermediateType::IU128,
        &u128::MIN.to_le_bytes(),
        &u128::MIN.abi_encode()
    )]
    #[case(
        IntermediateType::IU128,
        &(u128::MAX - 1).to_le_bytes(),
        &(u128::MAX - 1).abi_encode()
    )]
    #[case(
        IntermediateType::IU256,
        &U256::from(256256256256u128).to_le_bytes::<32>(),
        &U256::from(256256256256u128).abi_encode()
    )]
    #[case(
        IntermediateType::IU256,
        &U256::MAX.to_le_bytes::<32>(),
        &U256::MAX.abi_encode()
    )]
    #[case(
        IntermediateType::IU256,
        &U256::MIN.to_le_bytes::<32>(),
        &U256::MIN.abi_encode()
    )]
    #[case(
        IntermediateType::IU256,
        &(U256::MAX - U256::from(1)).to_le_bytes::<32>(),
        &(U256::MAX - U256::from(1)).abi_encode()
    )]
    #[case(
        IntermediateType::IAddress,
        &address!("0x0000000000000000000000000000000000000000").abi_encode(),
        &address!("0x0000000000000000000000000000000000000000").abi_encode()
    )]
    #[case(
        IntermediateType::IAddress,
        &address!("0x1234567890abcdef1234567890abcdef12345678").abi_encode(),
        &address!("0x1234567890abcdef1234567890abcdef12345678").abi_encode()
    )]
    #[case(
        IntermediateType::IAddress,
        &address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").abi_encode(),
        &address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").abi_encode()
    )]
    #[case(
        IntermediateType::IAddress,
        &address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE").abi_encode(),
        &address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE").abi_encode()
    )]
    fn test_pack_uint(
        #[case] int_type: impl Packable,
        #[case] data: &[u8],
        #[case] expected_result: &[u8],
    ) {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(data.len() as i32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(int_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        // Args data should already be stored in memory
        int_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                writer_pointer, // unused for this type
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        // the return is the pointer to the packed value
        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_pack_u128_fuzz() {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate 16 bytes for u128
        func_body.i32_const(16);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(32); // ABI encoded size
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        IntermediateType::IU128
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                writer_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<u128>()
            .cloned()
            .for_each(|value: u128| {
                // Write value to memory (little-endian)
                let data = value.to_le_bytes();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed u128 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_u256_fuzz() {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate 32 bytes for u256
        func_body.i32_const(32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(32); // ABI encoded size
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        IntermediateType::IU256
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                writer_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<[u8; 32]>()
            .cloned()
            .for_each(|bytes: [u8; 32]| {
                let store = store.clone();
                let entrypoint = entrypoint.clone();
                let reset_memory = reset_memory.clone();

                let value = U256::from_le_bytes(bytes);

                // Write value to memory (little-endian)
                memory.write(&mut *store.0.borrow_mut(), 0, &bytes).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed U256 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_address_fuzz() {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate 32 bytes for address (padded)
        func_body.i32_const(32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(32); // ABI encoded size
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        IntermediateType::IAddress
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                local,
                writer_pointer,
                writer_pointer,
                &compilation_ctx,
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<[u8; 20]>()
            .cloned()
            .for_each(|bytes: [u8; 20]| {
                let value = Address::from_slice(&bytes);

                // Write value to memory (padded to 32 bytes)
                let data = value.abi_encode();
                memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.abi_encode();
                assert_eq!(
                    result_memory_data, expected,
                    "Packed address did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

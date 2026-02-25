use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiOperationError},
    abi_types::unpacking::Unpackable,
    data::RuntimeErrorData,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::MemArg};

pub fn unpack_reference_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::UnpackReference.get_generic_function_name(compilation_ctx, &[itype])?;
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
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    match itype {
        // For immediates, allocate and store
        IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IBool => {
            let ptr_local = module.locals.add(walrus::ValType::I32);

            let data_size = itype.wasm_memory_data_size()?;
            builder
                .i32_const(data_size)
                .call(compilation_ctx.allocator)
                .local_tee(ptr_local);

            itype.add_unpack_instructions(
                None,
                &mut builder,
                module,
                None,
                Some(ValType::I32),
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
                Some(runtime_error_data),
                None,
            )?;

            builder.store(
                compilation_ctx.memory_id,
                itype.store_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            builder.local_get(ptr_local);
        }

        IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::ISigner
        | IntermediateType::IVector(_)
        | IntermediateType::IStruct { .. }
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => {
            // Heap types are handled in the add_unpack_instructions function so this case should be unreachable
            return Err(RuntimeFunctionError::from(AbiError::Unpack(
                AbiOperationError::UnhandledHeapTypeReference,
            )));
        }

        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            return Err(RuntimeFunctionError::from(AbiError::Unpack(
                AbiOperationError::RefInsideRef,
            )));
        }
        IntermediateType::ITypeParameter(_) => {
            return Err(RuntimeFunctionError::from(AbiError::Unpack(
                AbiOperationError::UnpackingGenericTypeParameter,
            )));
        }
    }

    Ok(function.finish(
        vec![reader_pointer, calldata_reader_pointer],
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
    use alloy_sol_types::{SolType, SolValue, sol};
    use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc, sync::Arc};
    use walrus::{FunctionBuilder, ValType};

    /// Test helper for unpacking reference types
    fn unpack_ref(data: &[u8], ref_type: IntermediateType, expected_memory_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, ctx_globals) =
            build_module(Some(data.len() as i32));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_memory_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_memory_bytes,
            "Heap memory at returned pointer does not match expected content"
        );
    }

    // ============================================================================
    // Reference Types - Simple Element Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u8() {
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IRef(Arc::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(88u8,));
        let expected = 88u8.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u16() {
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IRef(Arc::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(88u16,));
        let expected = 88u16.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u32() {
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IRef(Arc::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(88u32,));
        unpack_ref(&data, int_type.clone(), &88u32.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u64() {
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IRef(Arc::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(88u64,));
        unpack_ref(&data, int_type.clone(), &88u64.to_le_bytes());
    }

    // ============================================================================
    // Reference Types - Heap-Allocated Element Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u128() {
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IRef(Arc::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(123u128,));
        let expected = 123u128.to_le_bytes().to_vec();
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u256() {
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IRef(Arc::new(IntermediateType::IU256));

        let value = U256::from(123u128);
        let expected = value.to_le_bytes::<32>().to_vec();

        let data = SolType::abi_encode_params(&(value,));
        unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IAddress));

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        unpack_ref(&data, ref_type.clone(), &data);
    }

    // ============================================================================
    // Reference Types - Vector Elements
    // ============================================================================

    #[test]
    fn test_unpack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let vector_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU8,
        ))));

        let vec_data = vec![1u8, 2u8, 3u8, 4u8];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&4u32.to_le_bytes()); // length
        expected.extend(&4u32.to_le_bytes()); // capacity
        expected.extend(&1u8.to_le_bytes()); // first elem
        expected.extend(&2u8.to_le_bytes()); // second elem
        expected.extend(&3u8.to_le_bytes()); // third elem
        expected.extend(&4u8.to_le_bytes()); // fourth elem
        unpack_ref(&data, vector_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_vec_u128() {
        type SolType = sol!((uint128[],));
        let vector_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let vec_data = vec![1u128, 2u128, 3u128];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&3u32.to_le_bytes()); // length
        expected.extend(&3u32.to_le_bytes()); // capacity
        // pointers to heap elements
        expected.extend(&((INITIAL_MEMORY_OFFSET + 180) as u32).to_le_bytes());
        expected.extend(&((INITIAL_MEMORY_OFFSET + 196) as u32).to_le_bytes());
        expected.extend(&((INITIAL_MEMORY_OFFSET + 212) as u32).to_le_bytes());
        expected.extend(&1u128.to_le_bytes());
        expected.extend(&2u128.to_le_bytes());
        expected.extend(&3u128.to_le_bytes());

        unpack_ref(&data, vector_type.clone(), &expected);
    }

    // ============================================================================
    // Fuzz Tests - Simple Element Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_u8_fuzz() {
        type SolType = sol!((uint8,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU8));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<u8>()
            .cloned()
            .for_each(|value: u8| {
                let data = SolType::abi_encode_params(&(value,));
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 1];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.to_le_bytes();
                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref u8 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_u16_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU16));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<u16>()
            .cloned()
            .for_each(|value: u16| {
                let data = value.abi_encode();
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 2];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.to_le_bytes();
                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref u16 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_u32_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU32));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<u32>()
            .cloned()
            .for_each(|value: u32| {
                let data = value.abi_encode();
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.to_le_bytes();
                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref u32 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_u64_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU64));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<u64>()
            .cloned()
            .for_each(|value: u64| {
                let data = value.abi_encode();
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 8];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.to_le_bytes();
                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref u64 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_u128_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU128));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

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
                let data = value.abi_encode();
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 16];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.to_le_bytes();
                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref u128 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_u256_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU256));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

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
                let value = U256::from_le_bytes(bytes);
                let data = value.abi_encode();
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let expected = value.to_le_bytes::<32>();
                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref u256 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_address_fuzz() {
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IAddress));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 32], "test_function", None);

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
                let data = value.abi_encode();
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let mut result_memory_data = vec![0; 32];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, data,
                    "Unpacked ref address did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    // ============================================================================
    // Fuzz Tests - Vector Types
    // ============================================================================

    #[test]
    fn test_unpack_ref_vec_u8_fuzz() {
        type SolType = sol!((uint8[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU8,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(1024));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 1024], "test_function", None);

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
            .cloned()
            .for_each(|vec_data: Vec<u8>| {
                let data = SolType::abi_encode_params(&(vec_data.clone(),));
                if data.len() > 1024 {
                    return; // Skip if encoded data is too large
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = vec_data.len();
                let expected_size = 4 + 4 + len; // length + capacity + data
                let mut result_memory_data = vec![0; expected_size];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let mut expected: Vec<u8> = Vec::new();
                expected.extend(&(len as u32).to_le_bytes()); // length
                expected.extend(&(len as u32).to_le_bytes()); // capacity
                expected.extend(&vec_data); // data

                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref vec<u8> did not match expected result for len {len}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_vec_u16_fuzz() {
        type SolType = sol!((uint16[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU16,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(2048));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 2048], "test_function", None);

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
            .cloned()
            .for_each(|vec_data: Vec<u16>| {
                let data = SolType::abi_encode_params(&(vec_data.clone(),));
                if data.len() > 2048 {
                    return; // Skip if encoded data is too large
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = vec_data.len();
                let expected_size = 4 + 4 + (len * 2); // length + capacity + data
                let mut result_memory_data = vec![0; expected_size];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let mut expected: Vec<u8> = Vec::new();
                expected.extend(&(len as u32).to_le_bytes()); // length
                expected.extend(&(len as u32).to_le_bytes()); // capacity
                for val in &vec_data {
                    expected.extend(&val.to_le_bytes());
                }

                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref vec<u16> did not match expected result for len {len}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_vec_u32_fuzz() {
        type SolType = sol!((uint32[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU32,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(2048));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 2048], "test_function", None);

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
            .cloned()
            .for_each(|vec_data: Vec<u32>| {
                let data = SolType::abi_encode_params(&(vec_data.clone(),));
                if data.len() > 2048 {
                    return; // Skip if encoded data is too large
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = vec_data.len();
                let expected_size = 4 + 4 + (len * 4); // length + capacity + data
                let mut result_memory_data = vec![0; expected_size];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let mut expected: Vec<u8> = Vec::new();
                expected.extend(&(len as u32).to_le_bytes()); // length
                expected.extend(&(len as u32).to_le_bytes()); // capacity
                for val in &vec_data {
                    expected.extend(&val.to_le_bytes());
                }

                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref vec<u32> did not match expected result for len {len}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_vec_u64_fuzz() {
        type SolType = sol!((uint64[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU64,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(2048));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 2048], "test_function", None);

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
            .cloned()
            .for_each(|vec_data: Vec<u64>| {
                let data = SolType::abi_encode_params(&(vec_data.clone(),));
                if data.len() > 2048 {
                    return; // Skip if encoded data is too large
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                let len = vec_data.len();
                let expected_size = 4 + 4 + (len * 8); // length + capacity + data
                let mut result_memory_data = vec![0; expected_size];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                let mut expected: Vec<u8> = Vec::new();
                expected.extend(&(len as u32).to_le_bytes()); // length
                expected.extend(&(len as u32).to_le_bytes()); // capacity
                for val in &vec_data {
                    expected.extend(&val.to_le_bytes());
                }

                assert_eq!(
                    result_memory_data, expected,
                    "Unpacked ref vec<u64> did not match expected result for len {len}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_ref_vec_u128_fuzz() {
        type SolType = sol!((uint128[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(4096));
        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type
            .add_unpack_instructions(
                Some(&ref_type),
                &mut func_body,
                &mut raw_module,
                None,
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

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 4096], "test_function", None);

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
            .cloned()
            .for_each(|vec_data: Vec<u128>| {
                // Limit vector size to avoid too large allocations
                if vec_data.len() > 50 {
                    return;
                }

                let data = SolType::abi_encode_params(&(vec_data.clone(),));
                if data.len() > 4096 {
                    return; // Skip if encoded data is too large
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                // For heap types, the vector stores pointers to the actual data
                let len = vec_data.len();
                let vector_header_size = 4 + 4 + (len * 4); // length + capacity + pointers
                let mut vector_data = vec![0; vector_header_size];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut vector_data,
                    )
                    .unwrap();

                // Verify length and capacity
                let stored_len = u32::from_le_bytes([
                    vector_data[0],
                    vector_data[1],
                    vector_data[2],
                    vector_data[3],
                ]);
                let stored_cap = u32::from_le_bytes([
                    vector_data[4],
                    vector_data[5],
                    vector_data[6],
                    vector_data[7],
                ]);
                assert_eq!(stored_len as usize, len, "Vector length mismatch");
                assert_eq!(stored_cap as usize, len, "Vector capacity mismatch");

                // Verify each element by following the pointers
                for (i, expected_val) in vec_data.iter().enumerate() {
                    let offset = 8 + (i * 4);
                    let ptr = u32::from_le_bytes([
                        vector_data[offset],
                        vector_data[offset + 1],
                        vector_data[offset + 2],
                        vector_data[offset + 3],
                    ]);

                    let mut element_data = vec![0; 16];
                    memory
                        .read(&mut *store.0.borrow_mut(), ptr as usize, &mut element_data)
                        .unwrap();

                    let stored_val = u128::from_le_bytes(element_data.try_into().unwrap());
                    assert_eq!(
                        stored_val, *expected_val,
                        "Element {i} mismatch: expected {expected_val}, got {stored_val}"
                    );
                }

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

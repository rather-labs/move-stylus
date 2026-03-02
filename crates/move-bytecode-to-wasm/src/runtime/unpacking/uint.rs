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

/// Generates a runtime function that unpacks a Solidity ABI `uint32` value.
///
/// The function reads a 32-byte ABI slot from calldata, extracts the low 4 bytes,
/// converts from big-endian to little-endian, returns the decoded `u32` value,
/// and advances the calldata reader pointer by the provided encoded size.
///
/// # WASM Function Arguments
/// * `reader_pointer` - (i32): pointer to the current ABI slot in calldata
/// * `encoded_size` - (i32): number of bytes to advance the calldata reader pointer
///
/// # WASM Function Returns
/// * `value` - (i32): decoded `u32` value
pub fn unpack_u32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::UnpackU32.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size = module.locals.add(ValType::I32);

    // Load the value
    builder
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
    builder
        .local_get(reader_pointer)
        .local_get(encoded_size)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    Ok(function.finish(vec![reader_pointer, encoded_size], &mut module.funcs))
}

/// Generates a runtime function that unpacks a Solidity ABI `uint64` value.
///
/// The function reads the low 8 bytes of a 32-byte ABI slot, converts from big-endian
/// to little-endian, returns the decoded `u64` value, and advances the calldata reader
/// pointer by one ABI word.
///
/// # WASM Function Arguments
/// * `reader_pointer` - (i32): pointer to the current ABI slot in calldata
///
/// # WASM Function Returns
/// * `value` - (i64): decoded `u64` value
pub fn unpack_u64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None, None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    let mut builder = function
        .name(RuntimeFunction::UnpackU64.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<64>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)?;

    // Load the value
    builder
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
    builder
        .local_get(reader_pointer)
        .i32_const(encoded_size as i32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::{FunctionBuilder, ValType};
    use wasmtime::WasmResults;

    use crate::{
        abi_types::unpacking::Unpackable,
        test_compilation_context,
        test_tools::{INITIAL_MEMORY_OFFSET, build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    /// Test helper for unpacking simple integer types that fit in WASM value types
    fn unpack_uint<T: WasmResults + PartialEq + std::fmt::Debug>(
        int_type: impl Unpackable,
        data: &[u8],
        expected_result: T,
        result_type: ValType,
    ) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        let mut function_builder = FunctionBuilder::new(&mut raw_module.types, &[], &[result_type]);

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
                Some(result_type),
                args_pointer,
                args_pointer,
                &compilation_ctx,
                None,
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module::<_, T>(&mut raw_module, data.to_vec(), "test_function", None);

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_unpack_u8() {
        type IntType = u8;
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IU8;

        let data = SolType::abi_encode_params(&(88,));
        unpack_uint(int_type.clone(), &data, 88, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u16() {
        type IntType = u16;
        type SolType = sol!((uint16,));
        let int_type = IntermediateType::IU16;

        let data = SolType::abi_encode_params(&(1616,));
        unpack_uint(int_type.clone(), &data, 1616, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u32() {
        type IntType = u32;
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IU32;

        let data = SolType::abi_encode_params(&(323232,));
        unpack_uint(int_type.clone(), &data, 323232, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i32, ValType::I32);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i32,
            ValType::I32,
        );
    }

    #[test]
    fn test_unpack_u64() {
        type IntType = u64;
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IU64;

        let data = SolType::abi_encode_params(&(6464646464,));
        unpack_uint(int_type.clone(), &data, 6464646464i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        unpack_uint(int_type.clone(), &data, IntType::MAX as i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        unpack_uint(int_type.clone(), &data, IntType::MIN as i64, ValType::I64);

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        unpack_uint(
            int_type.clone(),
            &data,
            (IntType::MAX - 1) as i64,
            ValType::I64,
        );
    }

    #[test]
    fn test_unpack_u32_fuzz() {
        type SolType = sol!((uint32,));

        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_set(args_pointer);

        IntermediateType::IU32
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                None,
                Some(ValType::I32),
                args_pointer,
                args_pointer,
                &compilation_ctx,
                None,
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<_, i32>(&mut raw_module, vec![], "test_function", None);

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
                let data = SolType::abi_encode_params(&(value,));

                // Write the encoded data to memory at INITIAL_MEMORY_OFFSET
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                assert_eq!(
                    result, value as i32,
                    "Unpacked u32 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_u64_fuzz() {
        type SolType = sol!((uint64,));

        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I64]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_set(args_pointer);

        IntermediateType::IU64
            .add_unpack_instructions(
                None,
                &mut func_body,
                &mut raw_module,
                None,
                Some(ValType::I64),
                args_pointer,
                args_pointer,
                &compilation_ctx,
                None,
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<_, i64>(&mut raw_module, vec![], "test_function", None);

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
                let data = SolType::abi_encode_params(&(value,));

                // Write the encoded data to memory at INITIAL_MEMORY_OFFSET
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result: i64 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                assert_eq!(
                    result, value as i64,
                    "Unpacked u64 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

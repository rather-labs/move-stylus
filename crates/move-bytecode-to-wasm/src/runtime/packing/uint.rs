use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{MemArg, StoreKind},
};

/// Generates a WASM function that packs a u32 value into Solidity ABI format.
///
/// The function converts the value from little-endian to big-endian and left-pads
/// it to 32 bytes as required by the Solidity ABI specification.
///
/// # WASM Function Arguments:
/// * `value` (i32) - the u32 value to pack
/// * `writer_pointer` (i32) - pointer to the memory location where the packed value will be written
///
/// # WASM Function Returns:
/// * None - the result is written directly to memory at the writer_pointer location
pub fn pack_u32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Little-endian to Big-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::PackU32.name().to_owned())
        .func_body();

    let value = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Get writer pointer on stack
    builder.local_get(writer_pointer);

    // Load the value to the stack
    builder.local_get(value);

    // Little-endian to Big-endian
    builder.call(swap_i32_bytes_function);

    // Store at writer pointer (left-padded to 32 bytes)
    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // ABI is left-padded to 32 bytes
            offset: 28,
        },
    );

    Ok(function.finish(vec![value, writer_pointer], &mut module.funcs))
}

/// Generates a WASM function that packs a u64 value into Solidity ABI format.
///
/// The function converts the value from little-endian to big-endian and left-pads
/// it to 32 bytes as required by the Solidity ABI specification.
///
/// # WASM Function Arguments:
/// * `value` (i64) - the u64 value to pack
/// * `writer_pointer` (i32) - pointer to the memory location where the packed value will be written
///
/// # WASM Function Returns:
/// * None - the result is written directly to memory at the writer_pointer location
pub fn pack_u64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Little-endian to Big-endian
    let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None, None)?;
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I64, ValType::I32], &[]);

    let mut builder = function
        .name(RuntimeFunction::PackU64.name().to_owned())
        .func_body();

    let value = module.locals.add(ValType::I64);
    let writer_pointer = module.locals.add(ValType::I32);

    // Get writer pointer on stack
    builder.local_get(writer_pointer);

    // Load the value to the stack
    builder.local_get(value);

    // Little-endian to Big-endian
    builder.call(swap_i64_bytes_function);

    // Store at writer pointer (left-padded to 32 bytes)
    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I64 { atomic: false },
        MemArg {
            align: 0,
            // ABI is left-padded to 32 bytes
            offset: 24,
        },
    );

    Ok(function.finish(vec![value, writer_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolValue;
    use rstest::rstest;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::packing::Packable,
        data::RuntimeErrorData,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    enum Int {
        U32(u32),
        U64(u64),
    }

    #[rstest]
    #[case::u8_88(IntermediateType::IU8, Int::U32(88), 88i8.abi_encode())]
    #[case::u8_max(IntermediateType::IU8, Int::U32(u8::MAX as u32), (u8::MAX as u16).abi_encode())]
    #[case::u8_min(IntermediateType::IU8, Int::U32(u8::MIN as u32), (u8::MIN as u16).abi_encode())]
    #[case::u8_max_minus_1(IntermediateType::IU8, Int::U32((u8::MAX - 1) as u32), (u8::MAX as u16 - 1).abi_encode())]
    #[case::u16_1616(IntermediateType::IU16, Int::U32(1616), 1616u16.abi_encode())]
    #[case::u16_max(IntermediateType::IU16, Int::U32(u16::MAX as u32), u16::MAX.abi_encode())]
    #[case::u16_min(IntermediateType::IU16, Int::U32(u16::MIN as u32), u16::MIN.abi_encode())]
    #[case::u16_max_minus_1(IntermediateType::IU16, Int::U32((u16::MAX - 1) as u32), (u16::MAX - 1).abi_encode())]
    #[case::u32_323232(IntermediateType::IU32, Int::U32(323232), 323232u32.abi_encode())]
    #[case::u32_max(IntermediateType::IU32, Int::U32(u32::MAX), u32::MAX.abi_encode())]
    #[case::u32_min(IntermediateType::IU32, Int::U32(u32::MIN), u32::MIN.abi_encode())]
    #[case::u32_max_minus_1(IntermediateType::IU32, Int::U32(u32::MAX - 1), (u32::MAX - 1).abi_encode())]
    #[case::u64_6464646464(IntermediateType::IU64, Int::U64(6464646464), 6464646464u64.abi_encode())]
    #[case::u64_max(IntermediateType::IU64, Int::U64(u64::MAX), u64::MAX.abi_encode())]
    #[case::u64_min(IntermediateType::IU64, Int::U64(u64::MIN), u64::MIN.abi_encode())]
    #[case::u64_max_minus_1(IntermediateType::IU64, Int::U64(u64::MAX - 1), (u64::MAX - 1).abi_encode())]
    fn test_uint(
        #[case] int_type: impl Packable,
        #[case] literal: Int,
        #[case] expected_result: Vec<u8>,
    ) {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let local = match literal {
            Int::U32(literal) => {
                func_body.i32_const(literal as i32);
                raw_module.locals.add(ValType::I32)
            }
            Int::U64(literal) => {
                func_body.i64_const(literal as i64);
                raw_module.locals.add(ValType::I64)
            }
        };
        func_body.local_set(local);

        let writer_pointer = raw_module.locals.add(ValType::I32);

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
                Some(&mut runtime_error_data),
                None,
                Some(ValType::I32),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

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
    fn test_pack_u32_fuzz() {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let value = raw_module.locals.add(ValType::I32);

        let writer_pointer = raw_module.locals.add(ValType::I32);

        func_body
            .i32_const(
                IntermediateType::IU32
                    .encoded_size(&compilation_ctx)
                    .unwrap() as i32,
            )
            .call(alloc_function)
            .local_set(writer_pointer);

        IntermediateType::IU32
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                value,
                writer_pointer,
                writer_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
                Some(ValType::I32),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![value], &mut raw_module.funcs);
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
            .with_type::<u32>()
            .cloned()
            .for_each(|value: u32| {
                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), (value as i32,))
                    .unwrap();

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
                    "Packed u32 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_pack_u64_fuzz() {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I64], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let value = raw_module.locals.add(ValType::I64);

        let writer_pointer = raw_module.locals.add(ValType::I32);

        func_body
            .i32_const(
                IntermediateType::IU64
                    .encoded_size(&compilation_ctx)
                    .unwrap() as i32,
            )
            .call(alloc_function)
            .local_set(writer_pointer);

        IntermediateType::IU64
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                value,
                writer_pointer,
                writer_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
                Some(ValType::I32),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![value], &mut raw_module.funcs);
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
            .with_type::<u64>()
            .cloned()
            .for_each(|value: u64| {
                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), (value as i64,))
                    .unwrap();

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
                    "Packed u64 did not match expected result for value {value}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

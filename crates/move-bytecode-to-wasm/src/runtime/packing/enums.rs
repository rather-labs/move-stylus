use crate::{
    CompilationContext,
    data::RuntimeErrorData,
    error::RuntimeError,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn pack_enum_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::PackEnum.name().to_owned())
        .func_body();

    let enum_ptr = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    let value = module.locals.add(ValType::I32);

    // Little-endian to Big-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;

    builder.local_get(writer_pointer);

    // Read variant number from enum pointer
    builder
        .local_get(enum_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(value);

    // Check if the value is less than 255, if not, we trap since enums larger than that are not
    // supported in Solidity
    builder.i32_const(255).binop(BinaryOp::I32GtU).if_else(
        None,
        |then| {
            then.return_error(
                module,
                compilation_ctx,
                Some(ValType::I32),
                runtime_error_data,
                RuntimeError::EnumSizeTooLarge,
            );
        },
        |_| {},
    );

    // Convert to Big-endian
    builder.local_get(value).call(swap_i32_bytes_function);

    // Store the variant number at the writer pointer (left-padded to 32 bytes)
    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // ABI is left-padded to 32 bytes
            offset: 28,
        },
    );

    Ok(function.finish(vec![enum_ptr, writer_pointer], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_tools::{INITIAL_MEMORY_OFFSET, build_module};
    use crate::{
        test_compilation_context,
        test_tools::{assert_runtime_error, setup_wasmtime_module},
    };
    use rstest::rstest;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::FunctionBuilder;

    #[rstest]
    #[case(
        0,
        &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    ]
    #[case(
        1,
        &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1])
    ]
    fn test_pack_enum_variant(#[case] variant_number: u32, #[case] expected_calldata_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let enum_ptr = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate memory for the enum variant number (4 bytes)
        func_body.i32_const(4);
        func_body.call(allocator);
        func_body.local_set(enum_ptr);

        // Allocate memory for the packed output (32 bytes for ABI encoding)
        func_body.i32_const(32);
        func_body.call(allocator);
        func_body.local_set(writer_pointer);

        // Call pack_enum_function
        func_body.local_get(enum_ptr);
        func_body.local_get(writer_pointer);

        let pack_enum_func = pack_enum_function(
            &mut raw_module,
            &compilation_ctx,
            &mut RuntimeErrorData::new(),
        )
        .unwrap();
        func_body.call_runtime_function(
            &compilation_ctx,
            pack_enum_func,
            &RuntimeFunction::PackEnum,
            Some(ValType::I32),
        );

        // Return the writer pointer for reading the calldata back
        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // Prepare initial memory with the variant number (little-endian)
        let data = variant_number.to_le_bytes().to_vec();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_calldata_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_calldata_bytes,
            "Packed enum calldata did not match expected result"
        );
    }

    #[rstest]
    #[case(256)]
    #[case(u16::MAX as u32)]
    #[case(u32::MAX)]
    fn test_pack_enum_variant_too_large(
        #[case] variant_number: u32,
    ) -> Result<(), RuntimeFunctionError> {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let enum_ptr = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate memory for the enum variant number (4 bytes)
        func_body.i32_const(4);
        func_body.call(allocator);
        func_body.local_set(enum_ptr);

        // Allocate memory for the packed output (32 bytes for ABI encoding)
        func_body.i32_const(32);
        func_body.call(allocator);
        func_body.local_set(writer_pointer);

        // Call pack_enum_function
        func_body.local_get(enum_ptr);
        func_body.local_get(writer_pointer);

        let pack_enum_func = pack_enum_function(
            &mut raw_module,
            &compilation_ctx,
            &mut RuntimeErrorData::new(),
        )
        .unwrap();
        func_body.call_runtime_function(
            &compilation_ctx,
            pack_enum_func,
            &RuntimeFunction::PackEnum,
            Some(ValType::I32),
        );

        // Return the writer pointer for reading the calldata back
        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // Prepare initial memory with the variant number (little-endian)
        let data = variant_number.to_le_bytes().to_vec();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, "test_function", None);

        let _: i32 = entrypoint.call(&mut store, ()).unwrap();

        assert_runtime_error(&mut store, &instance, RuntimeError::EnumSizeTooLarge);

        Ok(())
    }

    #[test]
    fn test_pack_enum_variant_fuzz() {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let enum_ptr = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate memory for the enum variant number (4 bytes)
        func_body.i32_const(4);
        func_body.call(allocator);
        func_body.local_set(enum_ptr);

        // Allocate memory for the packed output (32 bytes for ABI encoding)
        func_body.i32_const(32);
        func_body.call(allocator);
        func_body.local_set(writer_pointer);

        // Call pack_enum_function
        func_body.local_get(enum_ptr);
        func_body.local_get(writer_pointer);

        let pack_enum_func = pack_enum_function(
            &mut raw_module,
            &compilation_ctx,
            &mut RuntimeErrorData::new(),
        )
        .unwrap();
        func_body.call(pack_enum_func);

        // Return the writer pointer for reading the calldata back
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
            .with_type::<u32>()
            .cloned()
            .for_each(|variant: u32| {
                // Write variant number to memory (little-endian)
                let data = variant.to_le_bytes();
                memory.write(&mut *store.0.borrow_mut(), INITIAL_MEMORY_OFFSET as usize, &data).unwrap();

                let mut store = store.0.borrow_mut();
                let result: Result<i32, _> = entrypoint.0.call(&mut *store, ());

                if variant <= 255 {
                    // Should succeed for values <= 255
                    match result {
                        Ok(result_ptr) => {
                            let mut result_memory_data = vec![0; 32];
                            memory
                                .read(
                                    &mut *store,
                                    result_ptr as usize,
                                    &mut result_memory_data,
                                )
                                .unwrap();

                            // Expected: 32 bytes with variant in big-endian at the end (left-padded)
                            let mut expected = [0u8; 32];
                            expected[31] = variant as u8;

                            assert_eq!(
                                result_memory_data, expected,
                                "Packed enum calldata did not match expected result for variant {variant}",
                            );
                        }
                        Err(_) => {
                            panic!("Expected success for variant {variant} but got error");
                        }
                    }
                } else {
                    // Should trap for values > 255
                    assert_runtime_error(&mut store, &instance, RuntimeError::EnumSizeTooLarge);
                }

                reset_memory.0.call(&mut *store, ()).unwrap();
            });
    }
}

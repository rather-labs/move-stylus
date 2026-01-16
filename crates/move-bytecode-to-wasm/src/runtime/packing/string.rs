use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

/// Generates a WASM function that packs a Move string into Solidity ABI string format.
///
/// Move strings are represented as structs containing a vector of u8 bytes. This function
/// unpacks the inner vector, allocates memory at the end of calldata with proper 32-byte
/// alignment, writes an offset at writer_pointer, then writes the string length and data.
///
/// The packed format consists of:
/// 1. An offset value at writer_pointer (32 bytes)
/// 2. The string length at the allocated location (32 bytes)
/// 3. The UTF-8 string data, padded to the next 32-byte boundary
///
/// # WASM Function Arguments:
/// * `string_pointer` (i32) - pointer to the Move string structure (contains pointer to inner vector)
/// * `writer_pointer` (i32) - pointer where the offset to the packed string should be written
/// * `calldata_reference_pointer` (i32) - reference point for calculating relative offsets
///
/// # WASM Function Returns:
/// * None - the result is written directly to memory, with an offset at writer_pointer and the string data at the end of calldata
pub fn pack_string_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx), None)?;

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );
    let mut builder = function
        .name(RuntimeFunction::PackString.name().to_owned())
        .func_body();

    let string_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    let data_pointer = module.locals.add(ValType::I32);
    let vector_pointer = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);
    let reference_value = module.locals.add(ValType::I32);

    // String in Move has the following layout:
    // public struct String has copy, drop, store {
    //   bytes: vector<u8>,
    // }
    //
    // So we need to perform a load first to get to the inner vector
    builder
        .local_get(string_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(vector_pointer);

    // Load the length
    builder
        .local_get(vector_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Allocate space for the text, padding by 32 bytes plus 32 bytes for the length
    // Calculate: ((len + 31) & !31) + 32
    builder
        .local_get(len)
        .i32_const(31)
        .binop(BinaryOp::I32Add)
        .i32_const(!31)
        .binop(BinaryOp::I32And)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_set(data_pointer);

    // The value stored at this param position should be the distance from the start of this
    // calldata portion to the pointer
    builder
        .local_get(data_pointer)
        .local_get(calldata_reference_pointer)
        .binop(BinaryOp::I32Sub)
        .local_set(reference_value);

    // Write the offset at writer_pointer
    builder
        .local_get(reference_value)
        .local_get(writer_pointer)
        .call(pack_u32_function);

    // Set the vector pointer to point to the first element (skip vector header)
    builder
        .skip_vec_header(vector_pointer)
        .local_set(vector_pointer);

    // Write the length at data_pointer
    builder
        .local_get(len)
        .local_get(data_pointer)
        .call(pack_u32_function);

    // Increment the data pointer to point to the data area
    builder
        .local_get(data_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_set(data_pointer);

    builder
        .local_get(data_pointer)
        .local_get(vector_pointer)
        .local_get(len)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    Ok(function.finish(
        vec![string_pointer, writer_pointer, calldata_reference_pointer],
        &mut module.funcs,
    ))
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
        runtime::RuntimeFunction,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
    };

    #[rstest]
    #[case::empty("", "".abi_encode())]
    #[case::short("hello", "hello".abi_encode())]
    #[case::medium("Hello, World!", "Hello, World!".abi_encode())]
    #[case::long("This is a longer string that will test padding and multiple 32-byte chunks", "This is a longer string that will test padding and multiple 32-byte chunks".abi_encode())]
    #[case::exactly_32_bytes("12345678901234567890123456789012", "12345678901234567890123456789012".abi_encode())]
    #[case::exactly_31_bytes("1234567890123456789012345678901", "1234567890123456789012345678901".abi_encode())]
    #[case::exactly_33_bytes("123456789012345678901234567890123", "123456789012345678901234567890123".abi_encode())]
    #[case::special_characters("Hello\nWorld\tTest\x00", "Hello\nWorld\tTest\x00".abi_encode())]
    #[case::unicode("Hello 世界 🌍", "Hello 世界 🌍".abi_encode())]
    #[case::multiple_chunks("This string is long enough to require multiple 32-byte chunks when encoded according to Solidity ABI encoding rules", "This string is long enough to require multiple 32-byte chunks when encoded according to Solidity ABI encoding rules".abi_encode())]
    fn test_string_packing(#[case] input_string: &str, #[case] expected_result: Vec<u8>) {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let string_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Create Move string structure in memory:
        // 1. Allocate vector (length + capacity + data)
        let string_bytes = input_string.as_bytes();
        let len = string_bytes.len() as u32;
        let vector_size = 8 + len; // 4 bytes length + 4 bytes capacity + data
        func_body.i32_const(vector_size as i32);
        func_body.call(alloc_function);
        let vector_pointer = raw_module.locals.add(ValType::I32);
        func_body.local_set(vector_pointer);

        // Write vector length
        func_body.local_get(vector_pointer);
        func_body.i32_const(len as i32);
        func_body.store(
            memory_id,
            walrus::ir::StoreKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Write vector capacity
        func_body.local_get(vector_pointer);
        func_body.i32_const(len as i32);
        func_body.store(
            memory_id,
            walrus::ir::StoreKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 0,
                offset: 4,
            },
        );

        // Write string bytes
        for (i, &byte) in string_bytes.iter().enumerate() {
            func_body.local_get(vector_pointer);
            func_body.i32_const(byte as i32);
            func_body.store(
                memory_id,
                walrus::ir::StoreKind::I32_8 { atomic: false },
                walrus::ir::MemArg {
                    align: 0,
                    offset: 8 + i as u32,
                },
            );
        }

        // 2. Allocate String struct (4 bytes pointing to vector)
        func_body.i32_const(4);
        func_body.call(alloc_function);
        func_body.local_set(string_pointer);

        // Write vector pointer to String struct
        func_body.local_get(string_pointer);
        func_body.local_get(vector_pointer);
        func_body.store(
            memory_id,
            walrus::ir::StoreKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 0,
                offset: 0,
            },
        );

        // 3. Allocate space for packed output
        // String is dynamic, so we need space for offset (32 bytes) + data
        // The data will be allocated separately by pack_string_function
        func_body.i32_const(32); // Just space for the offset
        func_body.call(alloc_function);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        // 4. Call pack_string_function directly
        let pack_string_func = RuntimeFunction::PackString
            .get(&mut raw_module, Some(&compilation_ctx), None)
            .unwrap();
        func_body
            .local_get(string_pointer)
            .local_get(writer_pointer)
            .local_get(calldata_reference_pointer)
            .call(pack_string_func);

        // Return the writer pointer
        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        // Call the function
        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        // Read the packed result from memory
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_string_packing_fuzz() {
        use alloy_sol_types::SolValue;

        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);

        // Build a function that takes string length and returns writer pointer
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let string_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);
        let vector_pointer = raw_module.locals.add(ValType::I32);
        let len_param = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Get length parameter
        func_body.local_get(len_param);
        func_body.i32_const(8);
        func_body.binop(walrus::ir::BinaryOp::I32Add);
        func_body.call(alloc_function);
        func_body.local_set(vector_pointer);

        // Allocate String struct
        func_body.i32_const(4);
        func_body.call(alloc_function);
        func_body.local_set(string_pointer);

        // Write vector pointer to String struct
        func_body.local_get(string_pointer);
        func_body.local_get(vector_pointer);
        func_body.store(
            memory_id,
            walrus::ir::StoreKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Allocate space for packed output
        func_body.i32_const(32);
        func_body.call(alloc_function);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        // Call pack_string_function
        let pack_string_func = RuntimeFunction::PackString
            .get(&mut raw_module, Some(&compilation_ctx), None)
            .unwrap();
        func_body
            .local_get(string_pointer)
            .local_get(writer_pointer)
            .local_get(calldata_reference_pointer)
            .call(pack_string_func);

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![len_param], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<String>()
            .for_each(|input_string: &String| {
                let string_bytes = input_string.as_bytes();
                let len = string_bytes.len() as u32;

                // Write vector data (length + capacity + data) at memory offset 0
                let mut vector_data = vec![];
                vector_data.extend(&len.to_le_bytes());
                vector_data.extend(&len.to_le_bytes());
                vector_data.extend(string_bytes);

                memory
                    .write(&mut *store.0.borrow_mut(), 0, &vector_data)
                    .unwrap();

                let result_ptr: i32 = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), len as i32)
                    .unwrap();

                let expected = input_string.abi_encode();
                let mut result_memory_data = vec![0; expected.len()];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut result_memory_data,
                    )
                    .unwrap();

                assert_eq!(
                    result_memory_data, expected,
                    "Packed string did not match expected result for value {input_string}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

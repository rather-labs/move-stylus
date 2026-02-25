use crate::{
    CompilationContext,
    data::RuntimeErrorData,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

pub fn unpack_string_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Big-endian to Little-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;
    // Validate that the pointer fits in 32 bits
    let validate_pointer_fn = RuntimeFunction::ValidatePointer32Bit.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::UnpackString.name().to_owned())
        .func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    let data_reader_pointer = module.locals.add(ValType::I32);

    // The ABI encoded value of a dynamic type is a reference to the location of the
    // values in the call data.
    builder.local_get(reader_pointer).call_runtime_function(
        compilation_ctx,
        validate_pointer_fn,
        &RuntimeFunction::ValidatePointer32Bit,
        Some(ValType::I32),
    );

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
        .local_get(calldata_reader_pointer)
        .binop(BinaryOp::I32Add)
        .local_set(data_reader_pointer); // This references the vector actual data

    // Advance the reader pointer by 32
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
                // Abi encoded value is Big endian
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

    // Allocate space for the vector
    // Each u8 element takes 1 byte
    let allocate_vector_with_header_function =
        RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx), None)?;
    builder
        .local_get(length)
        .local_get(length)
        .i32_const(1)
        .call(allocate_vector_with_header_function)
        .local_set(vector_pointer);

    builder.local_get(vector_pointer).local_set(writer_pointer);

    // Set writer pointer to the start of the vector data
    builder
        .skip_vec_header(writer_pointer)
        .local_set(writer_pointer);

    // Copy elements
    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);

    builder.loop_(None, |loop_block| {
        let loop_block_id = loop_block.id();

        loop_block.local_get(writer_pointer);

        loop_block.local_get(data_reader_pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        loop_block.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Increment data reader pointer by 1 byte to point to the next u8 element
        loop_block
            .local_get(data_reader_pointer)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(data_reader_pointer);

        // Increment writer pointer by 1 byte to point to the next u8 element
        loop_block
            .local_get(writer_pointer)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(writer_pointer);

        // Increment i
        loop_block
            .local_get(i)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_tee(i);

        loop_block
            .local_get(length)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_block_id);
    });

    let struct_ptr = module.locals.add(ValType::I32);
    // Create the struct pointing to the vector
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(struct_ptr);

    // Save the vector pointer as the first value
    builder.local_get(vector_pointer).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Return the String struct
    builder.local_get(struct_ptr);

    Ok(function.finish(
        vec![reader_pointer, calldata_reader_pointer],
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
        data::RuntimeErrorData,
        runtime::RuntimeFunction,
        test_compilation_context,
        test_tools::{INITIAL_MEMORY_OFFSET, build_module, setup_wasmtime_module},
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
    fn test_string_unpacking(#[case] expected_string: &str, #[case] abi_encoded: Vec<u8>) {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let reader_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Set reader pointer to start of memory
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(reader_pointer);
        func_body.local_set(calldata_reference_pointer);

        // Call unpack_string_function
        let unpack_string_func = RuntimeFunction::UnpackString
            .get(&mut raw_module, Some(&compilation_ctx), Some(&mut runtime_error_data))
            .unwrap();
        func_body
            .local_get(reader_pointer)
            .local_get(calldata_reference_pointer)
            .call(unpack_string_func);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, abi_encoded, "test_function", None);

        // Call the function - returns pointer to String struct
        let string_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Read the vector pointer from the String struct
        let mut vector_ptr_bytes = [0u8; 4];
        memory
            .read(&mut store, string_ptr as usize, &mut vector_ptr_bytes)
            .unwrap();
        let vector_ptr = i32::from_le_bytes(vector_ptr_bytes);

        // Read the length from the vector (first 4 bytes)
        let mut len_bytes = [0u8; 4];
        memory
            .read(&mut store, vector_ptr as usize, &mut len_bytes)
            .unwrap();
        let len = i32::from_le_bytes(len_bytes);

        // Read the string data (skip 8 bytes for length + capacity)
        let mut string_bytes = vec![0u8; len as usize];
        memory
            .read(&mut store, (vector_ptr + 8) as usize, &mut string_bytes)
            .unwrap();

        let result_string = String::from_utf8(string_bytes).unwrap();
        assert_eq!(result_string, expected_string);
    }

    #[test]
    fn test_string_unpacking_fuzz() {
        let (mut raw_module, alloc_function, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let compilation_ctx =
            test_compilation_context!(memory_id, alloc_function, calldata_reader_pointer_global);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let reader_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Set reader pointer to start of memory
        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(reader_pointer);
        func_body.local_set(calldata_reference_pointer);

        // Call unpack_string_function
        let unpack_string_func = RuntimeFunction::UnpackString
            .get(&mut raw_module, Some(&compilation_ctx), Some(&mut runtime_error_data))
            .unwrap();
        func_body
            .local_get(reader_pointer)
            .local_get(calldata_reference_pointer)
            .call(unpack_string_func);

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
            .with_type::<String>()
            .for_each(|input_string: &String| {
                let abi_encoded = input_string.abi_encode();

                // Write the encoded data to memory at INITIAL_MEMORY_OFFSET
                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &abi_encoded,
                    )
                    .unwrap();

                let string_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                // Read the vector pointer from the String struct
                let mut vector_ptr_bytes = [0u8; 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        string_ptr as usize,
                        &mut vector_ptr_bytes,
                    )
                    .unwrap();
                let vector_ptr = i32::from_le_bytes(vector_ptr_bytes);

                // Read the length from the vector (first 4 bytes)
                let mut len_bytes = [0u8; 4];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        vector_ptr as usize,
                        &mut len_bytes,
                    )
                    .unwrap();
                let len = i32::from_le_bytes(len_bytes);

                // Read the string data (skip 8 bytes for length + capacity)
                let mut string_bytes = vec![0u8; len as usize];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        (vector_ptr + 8) as usize,
                        &mut string_bytes,
                    )
                    .unwrap();

                let result_string = String::from_utf8(string_bytes).unwrap();
                assert_eq!(
                    result_string, *input_string,
                    "Unpacked string did not match expected result for value {input_string}",
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }
}

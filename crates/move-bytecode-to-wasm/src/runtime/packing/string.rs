use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn pack_string_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx))?;

    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );
    let mut function_body = function_builder.func_body();

    let string_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    let data_pointer = module.locals.add(ValType::I32);
    let vector_pointer = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);
    let reference_value = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);

    // String in Move has the following layout:
    // public struct String has copy, drop, store {
    //   bytes: vector<u8>,
    // }
    //
    // So we need to perform a load first to get to the inner vector
    function_body
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
    function_body
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
    function_body
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
    function_body
        .local_get(data_pointer)
        .local_get(calldata_reference_pointer)
        .binop(BinaryOp::I32Sub)
        .local_set(reference_value);

    // Write the offset at writer_pointer
    function_body
        .local_get(reference_value)
        .local_get(writer_pointer)
        .call(pack_u32_function);

    // Set the vector pointer to point to the first element (skip vector header)
    function_body
        .local_get(vector_pointer)
        .i32_const(8)
        .binop(BinaryOp::I32Add)
        .local_set(vector_pointer);

    // Write the length at data_pointer
    function_body
        .local_get(len)
        .local_get(data_pointer)
        .call(pack_u32_function);

    // Increment the data pointer to point to the data area
    function_body
        .local_get(data_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_set(data_pointer);

    // Outer block: if the vector length is 0, we skip to the end
    function_body.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if length == 0
        outer_block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(outer_block_id);

        // Loop through the vector values
        outer_block.i32_const(0).local_set(i);
        outer_block.loop_(None, |loop_block| {
            let loop_block_id = loop_block.id();

            // Load byte from vector and store at data_pointer
            loop_block
                .local_get(data_pointer)
                .local_get(vector_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32_8 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Increment the vector pointer by 1 byte
            loop_block
                .local_get(vector_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(vector_pointer);

            // Increment the data pointer by 1 byte
            loop_block
                .local_get(data_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(data_pointer);

            // Increment i
            loop_block
                .local_get(i)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_tee(i);

            // Continue loop if i < len
            loop_block
                .local_get(len)
                .binop(BinaryOp::I32LtU)
                .br_if(loop_block_id);
        });
    });

    function_builder.name(RuntimeFunction::PackString.name().to_owned());
    Ok(function_builder.finish(
        vec![string_pointer, writer_pointer, calldata_reference_pointer],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        runtime::RuntimeFunction,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
    };

    fn test_string_packing(input_string: &str, expected_result: &[u8]) {
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
            .get(&mut raw_module, Some(&compilation_ctx))
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
    fn test_pack_string_empty() {
        type SolType = sol!((string,));
        let input = "";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_short() {
        type SolType = sol!((string,));
        let input = "hello";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_medium() {
        type SolType = sol!((string,));
        let input = "Hello, World!";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_long() {
        type SolType = sol!((string,));
        let input = "This is a longer string that will test padding and multiple 32-byte chunks";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_exactly_32_bytes() {
        type SolType = sol!((string,));
        let input = "12345678901234567890123456789012"; // exactly 32 bytes
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_exactly_31_bytes() {
        type SolType = sol!((string,));
        let input = "1234567890123456789012345678901"; // exactly 31 bytes
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_exactly_33_bytes() {
        type SolType = sol!((string,));
        let input = "123456789012345678901234567890123"; // exactly 33 bytes
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_special_characters() {
        type SolType = sol!((string,));
        // Use a string with special characters (avoiding invalid hex escapes)
        let input = "Hello\nWorld\tTest\x00";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_unicode() {
        type SolType = sol!((string,));
        let input = "Hello 世界 🌍";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }

    #[test]
    fn test_pack_string_multiple_chunks() {
        type SolType = sol!((string,));
        let input = "This string is long enough to require multiple 32-byte chunks when encoded according to Solidity ABI encoding rules";
        let expected_result = SolType::abi_encode_params(&(input,));
        test_string_packing(input, &expected_result);
    }
}

use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, runtime::RuntimeFunction};

pub const ERROR_SELECTOR: [u8; 4] = [0x08, 0xc3, 0x79, 0xa0];

/// Builds an error message with the error code converted to decimal.
///
/// This function takes a u64 error code from the stack and creates a structured error message
/// in the format: [length: u32 LE][selector: 4 bytes][head: 32 bytes][length: 32 bytes][message: variable]
/// where the message contains only the decimal representation of the error code.
///
/// # Arguments
/// - `builder`: WASM instruction sequence builder
/// - `module`: WASM module being built
/// - `compilation_ctx`: Compilation context with memory and allocator info
///
/// # Stack Input
/// - Expects a u64 (error code) on the top of the stack
///
/// # Returns
/// - `LocalId`: Pointer to the allocated error message blob
///
/// # Memory Layout
/// The returned blob has the following structure:
/// - Byte 0: Total message length (little-endian u8)
/// - Bytes 1-4: Error selector (4 bytes)
/// - Bytes 5-36: Head word (32 bytes, with 0x20 at offset 35)
/// - Bytes 37-68: Length word (32 bytes, big-endian message length at offset 64)
/// - Bytes 69+: Error message text (e.g., "123")
pub fn build_error_message(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> LocalId {
    let ptr = module.locals.add(ValType::I32);

    // Convert error code to decimal string
    let u64_to_dec_ascii = RuntimeFunction::U64ToAsciiBase10.get(module, Some(compilation_ctx));
    let error_ptr = module.locals.add(ValType::I32);
    let error_len = module.locals.add(ValType::I32);

    // Convert u64 error code to decimal ASCII string
    builder.call(u64_to_dec_ascii).local_tee(error_ptr);

    // Load the length of the decimal string from memory (1 byte)
    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(error_len);

    // Skip length header to get pointer to actual error code string
    builder
        .local_get(error_ptr)
        .i32_const(1)
        .binop(BinaryOp::I32Add)
        .local_set(error_ptr);

    // Step 1: Calculate memory requirements
    let total_len = module.locals.add(ValType::I32);
    let padded_error_len = module.locals.add(ValType::I32);

    // Round up message length to 32-byte boundary for ABI alignment
    builder
        .local_get(error_len)
        .i32_const(31)
        .binop(BinaryOp::I32Add)
        .i32_const(!31)
        .binop(BinaryOp::I32And)
        .local_set(padded_error_len);

    // Calculate total allocation: header(68) + padded_error
    const ABI_HEADER_SIZE: i32 = 4 + 32 + 32; // selector(4) + head(32) + length(32)
    builder
        .i32_const(ABI_HEADER_SIZE)
        .local_get(padded_error_len)
        .binop(BinaryOp::I32Add)
        .local_set(total_len);

    // Allocate memory
    // We allocate 1 + total_len bytes to store the total length of the message plus the memory for the ABI-encoded error message.
    builder
        .i32_const(1) // 1 byte for the length
        .local_get(total_len) // total length of the message
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_tee(ptr)
        .local_get(total_len)
        .store(
            compilation_ctx.memory_id, // Store the total length in memory
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Write ABI header

    // Write error selector (bytes 1-4)
    for (i, b) in ERROR_SELECTOR.iter().enumerate() {
        builder.local_get(ptr).i32_const(*b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 1 + i as u32,
            },
        );
    }

    // Write head word
    const HEAD_WORD_END: u32 = 35; // last byte of the 32 bytes head word
    builder.local_get(ptr).i32_const(32).store(
        compilation_ctx.memory_id,
        StoreKind::I32_8 { atomic: false },
        MemArg {
            align: 0,
            offset: 1 + HEAD_WORD_END,
        },
    );

    // Write message length in big-endian format
    const LENGTH_WORD_END: u32 = 64; // last 4 bytes of the 32 bytes length word
    let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None);
    builder
        .local_get(ptr)
        .local_get(error_len)
        .call(swap_i32)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 1 + LENGTH_WORD_END,
            },
        );

    // Step 4: Write error message data
    builder
        .local_get(ptr)
        .i32_const(1 + ABI_HEADER_SIZE)
        .binop(BinaryOp::I32Add)
        .local_get(error_ptr)
        .local_get(error_len)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    ptr
}


#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use rstest::rstest;
    use walrus::FunctionBuilder;

    use super::*;

    #[rstest]
    #[case(0u64, "0")]
    #[case(1u64, "1")]
    #[case(123u64, "123")]
    #[case(999u64, "999")]
    #[case(1000u64, "1000")]
    #[case(999999u64, "999999")]
    #[case(1000000u64, "1000000")]
    #[case(123456789u64, "123456789")]
    #[case(9876543210u64, "9876543210")]
    #[should_panic]
    #[case(u64::MAX, "18446744073709551615")]
    fn test_build_error_message(#[case] error_code: u64, #[case] expected: &str) {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        // Create a test function that calls build_error_message
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I64], &[ValType::I32]);
        let n = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();
        func_body.i64_const(error_code as i64);
        let error_ptr = build_error_message(&mut func_body, &mut raw_module, &compilation_ctx);
        func_body.local_get(error_ptr);
        let function = function_builder.finish(vec![n], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i64, i32>(&mut raw_module, vec![], "test_function", None);

        let ptr = entrypoint.call(&mut store, 0).unwrap();

        // Read the error blob from the returned pointer
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let memory_data = memory.data(&mut store);

        // Read the total length (1 byte)
        let total_len = memory_data[ptr as usize] as u32;
        
        // Read the error selector (4 bytes at offset 1)
        let error_selector = memory_data[ptr as usize + 1..ptr as usize + 5].to_vec();
        assert_eq!(error_selector, ERROR_SELECTOR, "Error selector mismatch");

        // Read the head word (32 bytes at offset 5)
        let head_word = memory_data[ptr as usize + 5..ptr as usize + 37].to_vec();
        let mut expected_head_word = vec![0; 32];
        expected_head_word[31] = 0x20;
        assert_eq!(head_word, expected_head_word, "Head word mismatch");
        
        // Read the error message length from the ABI header (4 bytes big-endian at offset 65 = 1 + 4 + 32 + 32 - 4)
        let msg_len = u32::from_be_bytes([
            memory_data[ptr as usize + 65],
            memory_data[ptr as usize + 66],
            memory_data[ptr as usize + 67],
            memory_data[ptr as usize + 68],
        ]) as usize;

        // round up the msg_len to 32 bytes
        let padded_msg_len = (msg_len + 31) & !31;

        assert_eq!(total_len, padded_msg_len as u32 + 68, "Error message length mismatch");

        // Read the ASCII error message
        let error_start = ptr as usize + 69; // 1 + 4 + 32 + 32 = 69
        let error_message_data = &memory_data[error_start..error_start + msg_len];
        let result_str = String::from_utf8(error_message_data.to_vec()).unwrap();
        assert_eq!(result_str, expected, "Failed for input {}", error_code);
    }
}

use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::abi_encoding::{self, AbiFunctionSelector},
    abi_types::packing::build_pack_instructions,
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
    utils::snake_to_upper_camel,
};

use super::error::AbiError;

/// Error(string) abi encoded selector
pub const ERROR_SELECTOR: [u8; 4] = [0x08, 0xc3, 0x79, 0xa0];
const LENGTH_HEADER_SIZE: i32 = 4;

/// Calculate the error selector according to Solidity's [ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector)
///
/// Function names are converted to camel case before encoding.
pub fn move_signature_to_abi_selector(
    struct_name: &str,
    struct_fields: &[IntermediateType],
    compilation_ctx: &CompilationContext,
) -> Result<AbiFunctionSelector, AbiError> {
    abi_encoding::move_signature_to_abi_selector(
        struct_name,
        struct_fields,
        compilation_ctx,
        snake_to_upper_camel,
    )
}

/// Builds a custom error message from already ABI-encoded error parameters.
///
/// This function takes ABI-encoded error parameters (from `build_pack_instructions`) and
/// prepends the error selector to create a complete Solidity custom error message.
///
/// # Arguments
/// * `builder` - WASM instruction sequence builder
/// * `module` - WASM module being built
/// * `compilation_ctx` - Compilation context with memory and allocator info
/// * `error_struct` - Error struct to be encoded
/// * `error_struct_ptr` - Pointer to the error struct in memory
///
/// # Returns
/// * A local id variable containing the pointer to the allocated error message blob
pub fn build_custom_error_message(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    error_struct: &IStruct,
    error_struct_ptr: LocalId,
) -> Result<LocalId, AbiError> {
    let ptr = module.locals.add(ValType::I32);

    // Compute the error selector
    let error_selector = move_signature_to_abi_selector(
        &error_struct.identifier,
        &error_struct.fields,
        compilation_ctx,
    )?;

    // Allocate memory: 4 bytes for length + 4 bytes for the error selector
    const SELECTOR_SIZE: i32 = 4;
    builder
        .i32_const(LENGTH_HEADER_SIZE + SELECTOR_SIZE)
        .call(compilation_ctx.allocator)
        .local_set(ptr);

    // Write error selector after the length
    for (i, b) in error_selector.iter().enumerate() {
        builder.local_get(ptr).i32_const(*b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: (LENGTH_HEADER_SIZE + (i as i32)) as u32,
            },
        );
    }

    // Load each field to prepare them for ABI encoding.
    for (index, field) in error_struct.fields.iter().enumerate() {
        // Load each field's middle pointer
        builder.local_get(error_struct_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        // If the field is a stack type, load the value from memory
        if field.is_stack_type()? {
            builder.load(
                compilation_ctx.memory_id,
                field.load_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
    }

    // Combine all the error struct fields into one ABI-encoded error data buffer.
    let (_error_data_ptr, error_data_len) =
        build_pack_instructions(builder, &error_struct.fields, module, compilation_ctx)?;

    // Store the total length of the error message: length(4) + selector(4) + error_data_len
    builder
        .local_get(ptr)
        .local_get(error_data_len)
        .i32_const(SELECTOR_SIZE)
        .binop(BinaryOp::I32Add)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    Ok(ptr)
}

/// Builds an error message with the error code converted to decimal.
///
/// This function takes a u64 error code from the stack and creates a structured error message
/// in the format: [length: u32 LE][selector: 4 bytes][head: 32 bytes][length: 32 bytes][message: variable]
/// where the message contains only the decimal representation of the error code.
///
/// # Arguments
/// * `builder` - WASM instruction sequence builder
/// * `module` - WASM module being built
/// * `compilation_ctx` - Compilation context with memory and allocator info
///
/// # Returns
/// * A local id variable containing the pointer to the allocated error message blob
///
/// # Stack Input
/// * Expects a u64 (error code) on the top of the stack
///
/// # Memory Layout
/// The returned blob has the following structure:
/// * Bytes 0-3: Total message length (little-endian u32)
/// * Bytes 4-7: Error selector (4 bytes)
/// * Bytes 8-39: Head word (32 bytes, with 0x20 at offset 39)
/// * Bytes 40-71: Length word (32 bytes, big-endian message length at offset 68)
/// * Bytes 72+: Error message text (e.g., "123")
pub fn build_abort_error_message(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<LocalId, AbiError> {
    let ptr = module.locals.add(ValType::I32);

    // Convert error code to decimal string
    let u64_to_dec_ascii = RuntimeFunction::U64ToAsciiBase10.get(module, Some(compilation_ctx))?;
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
    // We allocate 4 + total_len bytes to store the total length of the message plus the memory for the ABI-encoded error message.
    builder
        .i32_const(LENGTH_HEADER_SIZE) // 4 bytes for the length
        .local_get(total_len) // total length of the message
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_tee(ptr)
        .local_get(total_len)
        .store(
            compilation_ctx.memory_id, // Store the total length in memory (4 bytes, little-endian u32)
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Write ABI header

    for (i, b) in ERROR_SELECTOR.iter().enumerate() {
        builder.local_get(ptr).i32_const(*b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: LENGTH_HEADER_SIZE as u32 + i as u32,
            },
        );
    }

    // Write head word
    // Head word is at offset 8-39 (32 bytes), last byte at offset 39
    // Relative to ABI header start (offset 4), head word is at 4-35, last byte at offset 35
    const HEAD_WORD_END: i32 = 35; // last byte of the 32 bytes head word (relative to ABI header start)
    builder.local_get(ptr).i32_const(32).store(
        compilation_ctx.memory_id,
        StoreKind::I32_8 { atomic: false },
        MemArg {
            align: 0,
            offset: (LENGTH_HEADER_SIZE + HEAD_WORD_END) as u32, // 4 + 35 = 39, which is the last byte of the head word
        },
    );

    // Write message length in big-endian format
    const LENGTH_WORD_END: i32 = 64; // last 4 bytes of the 32 bytes length word (offset 68 = 4 + 64)
    let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None)?;
    builder
        .local_get(ptr)
        .local_get(error_len)
        .call(swap_i32)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: (LENGTH_HEADER_SIZE + LENGTH_WORD_END) as u32,
            },
        );

    // Step 4: Write error message data
    builder
        .local_get(ptr)
        .i32_const(LENGTH_HEADER_SIZE + ABI_HEADER_SIZE)
        .binop(BinaryOp::I32Add)
        .local_get(error_ptr)
        .local_get(error_len)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    Ok(ptr)
}

#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use rstest::rstest;
    use walrus::{FunctionBuilder, ValType};

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
    #[case(u64::MAX, "18446744073709551615")]
    fn test_build_abort_error_message(#[case] error_code: u64, #[case] expected: &str) {
        let (mut raw_module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        // Create a test function that calls build_abort_error_message
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I64], &[ValType::I32]);
        let n = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();
        func_body.i64_const(error_code as i64);
        let error_ptr =
            build_abort_error_message(&mut func_body, &mut raw_module, &compilation_ctx).unwrap();
        func_body.local_get(error_ptr);
        let function = function_builder.finish(vec![n], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i64, i32>(&mut raw_module, vec![], "test_function", None);

        let ptr = entrypoint.call(&mut store, 0).unwrap();

        // Read the error blob from the returned pointer
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let memory_data = memory.data(&mut store);

        // Read the total length (4 bytes, little-endian u32)
        let total_len = u32::from_le_bytes([
            memory_data[ptr as usize],
            memory_data[ptr as usize + 1],
            memory_data[ptr as usize + 2],
            memory_data[ptr as usize + 3],
        ]);

        // Read the error selector (4 bytes at offset 4)
        let error_selector = memory_data[ptr as usize + 4..ptr as usize + 8].to_vec();
        assert_eq!(error_selector, ERROR_SELECTOR, "Error selector mismatch");

        // Read the head word (32 bytes at offset 8)
        let head_word = memory_data[ptr as usize + 8..ptr as usize + 40].to_vec();
        let mut expected_head_word = vec![0; 32];
        expected_head_word[31] = 0x20; // Last byte of the 32-byte head word
        assert_eq!(head_word, expected_head_word, "Head word mismatch");

        // Read the error message length from the ABI header (4 bytes big-endian at offset 68 = 4 + 4 + 32 + 32 - 4)
        let msg_len = u32::from_be_bytes([
            memory_data[ptr as usize + 68],
            memory_data[ptr as usize + 69],
            memory_data[ptr as usize + 70],
            memory_data[ptr as usize + 71],
        ]) as usize;

        // round up the msg_len to 32 bytes
        let padded_msg_len = (msg_len + 31) & !31;

        // Assert that the total length is the sum of the padded message length and the ABI header length
        // Header size = 4 (length) + 68 (ABI header: selector + head + length word) = 72
        assert_eq!(
            total_len,
            padded_msg_len as u32 + 68,
            "Error message length mismatch"
        );

        // Read the error message
        let error_start = ptr as usize + 72; // 4 + 4 + 32 + 32 = 72
        let error_message_data = &memory_data[error_start..error_start + msg_len];
        let result_str = String::from_utf8(error_message_data.to_vec()).unwrap();
        assert_eq!(result_str, expected, "Failed for input {error_code}");
    }
}

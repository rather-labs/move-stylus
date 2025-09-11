use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, runtime::RuntimeFunction};

pub const ERROR_NO_FUNCTION_MATCH_MSG: &[u8] =
    b"Entrypoint router error: function signature not found";

pub const ABORT_INSTRUCTION_REACHED_MSG: &[u8] = b"Abort instruction reached: error code ";

pub const ERROR_SELECTOR: [u8; 4] = [0x08, 0xc3, 0x79, 0xa0];

/// Builds an error message for abort instructions with the error code converted to decimal.
///
/// This function takes a u64 error code from the stack and creates a structured error message
/// in the format: [length: u32 LE][selector: 4 bytes][head: 32 bytes][length: 32 bytes][message: variable]
/// where the message contains a prefix followed by the decimal
/// representation of the error code.
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
/// - Bytes 0-3: Total message length (little-endian u32)
/// - Bytes 4-7: Error selector (4 bytes)
/// - Bytes 8-39: Head word (32 bytes, with 0x20 at offset 35)
/// - Bytes 40-71: Length word (32 bytes, big-endian message length at offset 64)
/// - Bytes 72+: Error message text ("Abort instruction reached: error code 123")
pub fn build_abort_error_message(
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

    // Load the length of the decimal string from memory
    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(error_len);

    // Skip length header to get pointer to actual string data
    builder
        .local_get(error_ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_set(error_ptr);

    // Calculate total message length: prefix + decimal string
    let msg_raw_len = module.locals.add(ValType::I32);
    builder
        .i32_const(ABORT_INSTRUCTION_REACHED_MSG.len() as i32)
        .local_get(error_len)
        .binop(BinaryOp::I32Add)
        .local_set(msg_raw_len);

    // Build error blob header with ABI formatting
    build_error_blob_header(builder, module, compilation_ctx, msg_raw_len, ptr);

    // Write error message prefix (bytes 72+)
    const MSG_START: u32 = 4 + 4 + 32 + 32; // total_len(4) + selector(4) + head(32) + length(32)
    for (i, &b) in ABORT_INSTRUCTION_REACHED_MSG.iter().enumerate() {
        builder.local_get(ptr).i32_const(b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: MSG_START + i as u32,
            },
        );
    }

    // Append decimal error code after the prefix
    builder
        .local_get(ptr)
        .i32_const(MSG_START as i32 + ABORT_INSTRUCTION_REACHED_MSG.len() as i32)
        .binop(BinaryOp::I32Add)
        .local_get(error_ptr)
        .local_get(error_len)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    ptr
}

/// Builds an error message for when a function signature is not found in the entrypoint router.
///
/// This function creates a structured error message in the format:
/// [length: u32 LE][selector: 4 bytes][head: 32 bytes][length: 32 bytes][message]
/// where the message contains "Entrypoint router error: function signature not found".
///
/// # Arguments
/// - `builder`: WASM instruction sequence builder
/// - `module`: WASM module being built
/// - `compilation_ctx`: Compilation context with memory and allocator info
///
/// # Returns
/// - `LocalId`: Pointer to the allocated error message blob
///
/// # Memory Layout
/// The returned blob has the following structure:
/// - Bytes 0-3: Total message length (little-endian u32)
/// - Bytes 4-7: Error selector (4 bytes)
/// - Bytes 8-39: Head word (32 bytes, with 0x20 at offset 35)
/// - Bytes 40-71: Length word (32 bytes, big-endian message length at offset 64)
/// - Bytes 72+: Error message text ("Entrypoint router error: function signature not found")
pub fn build_no_function_match_error_message(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> LocalId {
    let ptr = module.locals.add(ValType::I32);
    let msg_len = module.locals.add(ValType::I32);

    // Set message length to the size of the error message string
    builder
        .i32_const(ERROR_NO_FUNCTION_MATCH_MSG.len() as i32)
        .local_set(msg_len);

    // Build error blob header with ABI formatting
    build_error_blob_header(builder, module, compilation_ctx, msg_len, ptr);

    // Write error message text (bytes 72+)
    const MSG_START: u32 = 4 + 4 + 32 + 32; // total_len + selector + head + length
    for (i, &byte) in ERROR_NO_FUNCTION_MATCH_MSG.iter().enumerate() {
        builder.local_get(ptr).i32_const(byte as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: MSG_START + i as u32,
            },
        );
    }

    ptr
}

/// Allocates and initializes the ABI error blob header.
///
/// This function creates a structured error message header in the format:
/// [length: u32 LE][selector: 4 bytes][head: 32 bytes][length: 32 bytes]
/// where the message data will be written after this header.
///
/// Note: Two different lengths are stored:
/// - First 4 bytes: Total allocated length (for memory management)
/// - Length word (bytes 40-71): Raw message length (for ABI decoding)
///
/// # Arguments
/// - `builder`: WASM instruction sequence builder
/// - `module`: WASM module being built
/// - `compilation_ctx`: Compilation context with memory and allocator info
/// - `msg_len`: LocalId containing the raw message length (i32)
/// - `ptr`: LocalId where the allocated memory pointer will be stored (i32 OUT)
///
/// # Memory Layout
/// The allocated blob has the following structure:
/// - Bytes 0-3: Total message length (little-endian u32)
/// - Bytes 4-7: Error selector (4 bytes)
/// - Bytes 8-39: Head word (32 bytes, with 0x20 at offset 39)
/// - Bytes 40-71: Length word (32 bytes, big-endian message length at offset 68)
/// - Bytes 72+: Message data (to be written by caller)
fn build_error_blob_header(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    msg_len: LocalId,
    ptr: LocalId,
) {
    const MSG_START: i32 = 4 + 32 + 32; // selector + head + len
    const HEAD_OFFSET: u32 = 35; // last byte of 32B head word
    const LENGTH_OFFSET: u32 = 64; // last 4 bytes of 32B len word

    let padded_len = module.locals.add(ValType::I32);
    let total_len = module.locals.add(ValType::I32);

    // Round up message length to 32-byte boundary for ABI alignment
    builder
        .local_get(msg_len)
        .i32_const(31)
        .binop(BinaryOp::I32Add)
        .i32_const(!31)
        .binop(BinaryOp::I32And)
        .local_set(padded_len);

    // Calculate total allocation size: header + padded message
    builder
        .i32_const(MSG_START)
        .local_get(padded_len)
        .binop(BinaryOp::I32Add)
        .local_set(total_len);

    // Allocate memory and store pointer
    builder
        .local_get(total_len)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_tee(ptr);

    // Store total length in first 4 bytes (for memory management)
    builder.local_get(total_len).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Write error selector (bytes 4-7)
    for (i, b) in ERROR_SELECTOR.iter().enumerate() {
        builder.local_get(ptr).i32_const(*b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 4 + i as u32,
            },
        );
    }

    // Write head word with 0x20 offset (byte 39)
    builder.local_get(ptr).i32_const(32).store(
        compilation_ctx.memory_id,
        StoreKind::I32_8 { atomic: false },
        MemArg {
            align: 0,
            offset: 4 + HEAD_OFFSET,
        },
    );

    // Write message length in big-endian format (bytes 68-71)
    let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None);
    builder
        .local_get(ptr)
        .local_get(msg_len)
        .call(swap_i32)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4 + LENGTH_OFFSET,
            },
        );
}

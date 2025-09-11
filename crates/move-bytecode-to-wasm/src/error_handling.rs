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
    let data_ptr = module.locals.add(ValType::I32);

    // Convert error code to decimal string
    let u64_to_dec_ascii = RuntimeFunction::U64ToAsciiBase10.get(module, Some(compilation_ctx));
    let error_ptr = module.locals.add(ValType::I32);
    let error_len = module.locals.add(ValType::I32);

    // Convert error code to decimal string
    builder.call(u64_to_dec_ascii).local_tee(error_ptr);

    // Load the length of the decimal string
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

    // Load the pointer to the decimal string
    builder
        .local_get(error_ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_set(error_ptr);

    // Calculate message length and total allocation size
    let msg_raw_len = module.locals.add(ValType::I32);
    let msg_total_len = module.locals.add(ValType::I32);
    const HEAD_OFFSET: u32 = 35; // Position of head word (0x20)
    const LENGTH_OFFSET: u32 = 64; // Position of length word
    const MSG_START: u32 = 4 + 32 + 32; // selector(4) + head(32) + len(32)

    // msg_raw_len = ABORT_INSTRUCTION_REACHED_MSG.len() + decimal_len
    builder
        .i32_const(ABORT_INSTRUCTION_REACHED_MSG.len() as i32)
        .local_get(error_len)
        .binop(BinaryOp::I32Add)
        .local_set(msg_raw_len);

    // total_len = selector(4) + head(32) + len(32) + padded_msg_len
    builder
        .local_get(msg_raw_len)
        .i32_const(31)
        .binop(BinaryOp::I32Add)
        .i32_const(!31)
        .binop(BinaryOp::I32And)
        .i32_const(MSG_START as i32)
        .binop(BinaryOp::I32Add)
        .local_set(msg_total_len);

    // Allocate memory: 4 bytes (for the length) + msg_total_len bytes (for the message)
    builder
        .local_get(msg_total_len)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_tee(ptr);

    // Store data length in the first 4 bytes of the allocated memory
    builder.local_get(msg_total_len).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Skip header and set data_ptr
    builder
        .local_get(ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_set(data_ptr);

    // Write error selector (first 4 bytes)
    for (i, b) in ERROR_SELECTOR.iter().enumerate() {
        builder.local_get(data_ptr).i32_const(*b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: i as u32,
            },
        );
    }

    // Write head word (offset to data = 0x20) in the last byte of the 32-byte word
    builder.local_get(data_ptr).i32_const(32).store(
        compilation_ctx.memory_id,
        StoreKind::I32_8 { atomic: false },
        MemArg {
            align: 0,
            offset: HEAD_OFFSET,
        },
    );

    // Write length word (big-endian, in the LAST 4 bytes of the 32-byte word)
    let swap_i32 = RuntimeFunction::SwapI32Bytes.get(module, None);
    builder
        .local_get(data_ptr)
        .local_get(msg_raw_len)
        .call(swap_i32)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: LENGTH_OFFSET,
            },
        );

    // Write prefix
    for (i, &b) in ABORT_INSTRUCTION_REACHED_MSG.iter().enumerate() {
        builder.local_get(data_ptr).i32_const(b as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: MSG_START + i as u32,
            },
        );
    }

    // Append decimal digits after the prefix
    builder
        .local_get(data_ptr)
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
    let data_ptr = module.locals.add(ValType::I32);

    let msg_len = ERROR_NO_FUNCTION_MATCH_MSG.len() as i32;
    let padded_len = ((msg_len + 31) / 32) * 32;
    let data_start = 4 + 32 + 32;
    let total_len = data_start + padded_len;

    // Allocate error buffer and set the pointer
    builder
        .i32_const(total_len)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_tee(ptr);

    // Store the total length in the first 4 bytes of the allocated memory
    builder.i32_const(total_len).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Skip header and set data_ptr
    builder
        .local_get(ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_set(data_ptr);

    // Write error selector (4 bytes)
    for (i, &byte) in ERROR_SELECTOR.iter().enumerate() {
        builder.local_get(data_ptr).i32_const(byte as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: i as u32,
            },
        );
    }

    // Write offset (32) -> dynamic encoding
    builder.local_get(data_ptr).i32_const(32).store(
        compilation_ctx.memory_id,
        StoreKind::I32_8 { atomic: false },
        MemArg {
            align: 0,
            offset: data_start as u32 - 32 - 1,
        },
    );

    // Write error message length (big-endian)
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);
    builder
        .local_get(data_ptr)
        .i32_const(msg_len)
        .call(swap_i32_bytes_function)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: data_start as u32 - 4,
            },
        );

    // Write message data
    for (i, &byte) in ERROR_NO_FUNCTION_MATCH_MSG.iter().enumerate() {
        builder.local_get(data_ptr).i32_const(byte as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: data_start as u32 + i as u32,
            },
        );
    }

    ptr
}

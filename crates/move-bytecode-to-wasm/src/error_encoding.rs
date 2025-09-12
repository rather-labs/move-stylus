use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, ExtendedLoad},
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

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// Converts a non-negative 64-bit integer to its decimal ASCII representation.
///
/// Produces a blob: `[len: u32 LE][ASCII decimal bytes...]`.
///
/// # Arguments
/// - `val`: The value to convert. Passed as WASM `i64` but must be ≥ 0; traps if negative.
///          (Conceptually a `u64`.)
///
/// # Returns
/// - `ptr`: Pointer to the allocated blob `[len: u32 LE][ASCII bytes...]`.
///
/// # Memory Layout
/// - Bytes 0..4 : Little-endian `u32` length of the ASCII string
/// - Bytes 4..  : ASCII digits (e.g., "123" for value 123)
///
/// # Examples
/// - Input: 0          → `[1, 0, 0, 0, '0']`                      // len=1, data="0"
/// - Input: 123        → `[3, 0, 0, 0, '1','2','3']`               // len=3, data="123"
/// - Input: 999_999    → `[6, 0, 0, 0, '9','9','9','9','9','9']`   // len=6, data="999999"
///
/// Notes:
/// - Each decimal digit `d` (0..9) is encoded as the ASCII byte `'0' + d` (0x30..0x39).
pub fn u64_to_ascii_base_10(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I32]);

    // Function arguments and local variables
    let val = module.locals.add(ValType::I64); // input value (must be >= 0)
    let n = module.locals.add(ValType::I64); // working copy for digit extraction
    let len = module.locals.add(ValType::I32); // length of the resulting string
    let data_ptr = module.locals.add(ValType::I32); // pointer to the allocated blob

    let mut builder = function
        .name(RuntimeFunction::U64ToAsciiBase10.name().to_owned())
        .func_body();

    // Safety check: trap if the input value is negative
    // This should never happen for Move abort codes, but we check for safety
    builder
        .local_get(val)
        .i64_const(0)
        .binop(BinaryOp::I64LtS)
        .if_else(
            None,
            |t| {
                t.unreachable(); // Trap on negative input
            },
            |_| {},
        );

    // Step 1: Count the number of decimal digits needed
    // Algorithm: len = 1; n = val; while (n >= 10) { n /= 10; len++; }
    // This ensures we allocate exactly the right amount of memory
    builder.local_get(val).local_set(n); // n = val (working copy)
    builder.i32_const(1).local_set(len); // len = 1 (minimum for single digit)

    builder.block(None, |block| {
        let block_id = block.id();
        block.loop_(None, |lp| {
            let lp_id = lp.id();

            // Check if we've processed all digits: if (n < 10) break;
            lp.local_get(n)
                .i64_const(10)
                .binop(BinaryOp::I64LtU)
                .br_if(block_id);

            // Process next digit: n /= 10; len++;
            lp.local_get(n)
                .i64_const(10)
                .binop(BinaryOp::I64DivU)
                .local_set(n);

            // Increment digit counter
            lp.local_get(len)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(len);

            lp.br(lp_id); // Continue loop
        });
    });

    // Step 2: Allocate memory for the result blob
    // Memory layout: [4 bytes for length][len bytes for ASCII digits]
    builder
        .i32_const(4)
        .local_get(len)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_set(data_ptr);

    // Step 3: Store the length in the first 4 bytes (little-endian)
    builder.local_get(data_ptr).local_get(len).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Step 4: Set up write pointer for digit storage
    // We'll write digits backwards, so start at the end of the string
    let write_ptr = module.locals.add(ValType::I32);
    builder
        .local_get(data_ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_get(len)
        .binop(BinaryOp::I32Add)
        .local_set(write_ptr);

    // Step 5: Extract and write digits in reverse order
    // Algorithm: n = val; while (n != 0) { *--write_ptr = '0' + (n % 10); n /= 10; };
    // This writes digits from right to left, then we'll have the correct order
    let char_ = module.locals.add(ValType::I32);
    builder.local_get(val).local_set(n);

    builder.block(None, |block| {
        let block_id = block.id();
        block.loop_(None, |lp| {
            let lp_id = lp.id();

            // Extract the rightmost digit and convert to ASCII
            // char_ = '0' + (n % 10)
            lp.local_get(n)
                .i64_const(10)
                .binop(BinaryOp::I64RemU) // n % 10 (get rightmost digit)
                .i64_const(0x30) // ASCII '0' = 0x30
                .binop(BinaryOp::I64Add) // '0' + digit
                .unop(UnaryOp::I32WrapI64) // Convert i64 to i32
                .local_set(char_);

            // Write the character to memory (moving backwards)
            // *--write_ptr = char_
            lp.local_get(write_ptr)
                .i32_const(1)
                .binop(BinaryOp::I32Sub) // --write_ptr
                .local_tee(write_ptr) // Update write_ptr
                .local_get(char_) // Get the ASCII character
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32_8 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Remove the processed digit and check if we're done
            // n /= 10; if (n == 0) break;
            lp.local_get(n)
                .i64_const(10)
                .binop(BinaryOp::I64DivU) // n /= 10
                .local_set(n);
            lp.local_get(n)
                .i64_const(0)
                .binop(BinaryOp::I64Eq) // if (n == 0)
                .br_if(block_id); // break;

            lp.br(lp_id); // Continue loop
        });
    });

    // Step 6: Return the pointer to the complete blob
    // The blob contains [length][ASCII digits] and is ready to use
    builder.local_get(data_ptr);

    function.finish(vec![val], &mut module.funcs)
}

use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::CompilationContext;

use super::RuntimeFunction;

/// Converts a non-negative 64-bit integer to its decimal ASCII representation.
///
/// Produces a blob: `[len: u32 LE][ASCII decimal bytes...]`.
///
/// # Arguments
/// - `val`: The value to convert. Passed as WASM `i64` but must be ≥ 0; traps if negative.
///
/// # Returns
/// - `ptr`: Pointer to the allocated blob `[len: u32 LE][ASCII bytes...]`.
///
/// # Memory Layout
/// - Byte 0: Little-endian `u8` length of the ASCII string
/// - Bytes 1..  : ASCII digits (e.g., "123" for value 123)
///
/// # Examples
/// - Input: 0          → `[1, '0']`                       // len=1, data="0"
/// - Input: 123        → `[3, '1','2','3']`               // len=3, data="123"
/// - Input: 999_999    → `[6, '9','9','9','9','9','9']`   // len=6, data="999999"
///
/// Notes:
/// - Each decimal digit `d` (0..9) is encoded as the ASCII byte `'0' + d` (0x30..0x39).
pub fn u64_to_ascii_base_10(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    use walrus::ir::{BinaryOp, MemArg, StoreKind, UnaryOp};

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I32]);

    // locals
    let n = module.locals.add(ValType::I64); // input (>= 0)
    let len = module.locals.add(ValType::I32); // digit count
    let ptr = module.locals.add(ValType::I32); // [len|bytes..]
    let scale = module.locals.add(ValType::I64); // current power of 10

    let mut builder = function
        .name(RuntimeFunction::U64ToAsciiBase10.name().to_owned())
        .func_body();

    // trap if negative, or n > 10ˆ18
    builder
        .local_get(n)
        .i64_const(0)
        .binop(BinaryOp::I64LtS)
        .if_else(
            None,
            |t| {
                t.unreachable();
            },
            |_| {},
        );

    // Handle n = 0 case
    builder
        .local_get(n)
        .i64_const(0)
        .binop(BinaryOp::I64Eq)
        .if_else(
            None,
            |z| {
                z.i32_const(1)
                    .call(compilation_ctx.allocator)
                    .local_tee(ptr)
                    .i32_const(1)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32_8 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // store the '0' digit
                z.i32_const(1)
                    .call(compilation_ctx.allocator)
                    .i32_const(0x30)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32_8 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            },
            |nz| {
                // scale = 10^18;
                nz.i64_const(1_000_000_000_000_000_000i64).local_set(scale);

                // len = 19
                nz.i32_const(19).local_set(len);

                // allocate memory for the length
                nz.i32_const(1)
                    .call(compilation_ctx.allocator)
                    .local_set(ptr);

                // while (scale > n) { scale /= 10; len--; }
                nz.block(None, |block| {
                    let block_id = block.id();

                    block.loop_(None, |lp| {
                        let lp_id = lp.id();
                        lp.local_get(scale)
                            .local_get(n)
                            .binop(BinaryOp::I64LeU)
                            .br_if(block_id);

                        lp.local_get(scale)
                            .i64_const(10)
                            .binop(BinaryOp::I64DivU)
                            .local_set(scale);

                        lp.local_get(len)
                            .i32_const(1)
                            .binop(BinaryOp::I32Sub)
                            .local_set(len);
                        lp.br(lp_id);
                    });
                });

                // while (true) {
                //   digit = n / scale; *write_ptr++ = '0' + digit; n -= digit * scale;
                //   if (scale == 1) break; scale /= 10;
                // }
                nz.block(None, |block| {
                    let block_id = block.id();
                    let write_ptr = module.locals.add(ValType::I32);
                    block.loop_(None, |lp| {
                        let lp_id = lp.id();

                        // Allocate 1 byte for the digit
                        lp.i32_const(1)
                            .call(compilation_ctx.allocator)
                            .local_tee(write_ptr);

                        // digit = (n / scale) + '0'
                        lp.local_get(n)
                            .local_get(scale)
                            .binop(BinaryOp::I64DivU)
                            .i64_const(0x30)
                            .binop(BinaryOp::I64Add)
                            .unop(UnaryOp::I32WrapI64);

                        // store the digit
                        lp.store(
                            compilation_ctx.memory_id,
                            StoreKind::I32_8 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                        // n -= digit * scale
                        lp.local_get(n)
                            .local_get(scale)
                            .binop(BinaryOp::I64RemU)
                            .local_set(n);

                        // if (scale == 1) break;
                        lp.local_get(scale)
                            .i64_const(1)
                            .binop(BinaryOp::I64Eq)
                            .br_if(block_id);

                        // scale /= 10; continue
                        lp.local_get(scale)
                            .i64_const(10)
                            .binop(BinaryOp::I64DivU)
                            .local_set(scale);

                        lp.br(lp_id);
                    });
                });

                // Store the length
                nz.local_get(ptr).local_get(len).store(
                    compilation_ctx.memory_id,
                    StoreKind::I32_8 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            },
        );

    builder.local_get(ptr);

    function.finish(vec![n], &mut module.funcs)
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
    fn test_u64_to_ascii_base_10(#[case] error_code: u64, #[case] expected: &str) {
        let (mut raw_module, allocator_func, memory_id) = build_module(None);
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);

        // Add the u64_to_ascii_base_10 function to the module
        let ascii_func = u64_to_ascii_base_10(&mut raw_module, &compilation_ctx);

        // Create a test function that calls u64_to_ascii_base_10 and writes to memory
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I64], &[]);
        let n = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();
        func_body.i64_const(error_code as i64);
        func_body.call(ascii_func);
        func_body.drop();

        let function = function_builder.finish(vec![n], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i64, ()>(&mut raw_module, vec![], "test_function", None);

        entrypoint.call(&mut store, 0).unwrap();

        // Read the result from memory at offset 0
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let memory_data = memory.data(&mut store);

        let len = memory_data[0] as u32;

        // Read the ASCII string
        let ascii_data = &memory_data[1..1 + len as usize];
        let result_str = String::from_utf8(ascii_data.to_vec()).unwrap();

        assert_eq!(result_str, expected, "Failed for input {error_code}");
    }
}

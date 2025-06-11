use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// Implements the long multiplication algorithm for 128 and 256 bit integers.
///
/// The algorithm is inspired by the classic grade-school method, breaking down each 128-bit input
/// into four 32-bit chunks:
///
///    a4 a3 a2 a1
/// x  b4 b3 b2 b1
///
/// We use 32-bit chunks to allow for carry propagation, as carries can exceed a single bit.
///
/// First Iteration (multiplying by b1):
///
/// a1 x b1 = r1_1               → c1
/// a2 x b1 = r1_2 + carry(r1_1) → c2
/// a3 x b1 = r1_3 + carry(r1_2) → c3
/// a4 x b1 = r1_4 + carry(r1_3) → c4
///    → If there's a carry in this final step, it overflows.
///
/// Second Iteration (multiplying by b2):
///
/// a1 x b2 = r2_1               → d1
/// a2 x b2 = r2_2 + carry(r2_1) → d2
/// a3 x b2 = r2_3 + carry(r2_2) → d3
///    → If there's a carry here, it overflows.
///
/// ...and so on for b3 and b4.
///
/// Final Summation:
///
///    a4 a3 a2 a1
/// x  b4 b3 b2 b1
///    -----------
/// +  c4 c3 c2 c1
/// +  d3 d2 d1 0
/// +  e2 e1 0  0
/// +  f1 0  0  0
///
/// This approach allows us to optimize both carry detection and performance.
/// In terms of memory, we only need to allocate space for the final result, as intermediate
/// computations are performed in place within the result buffer.
pub fn heap_integers_mul(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::HeapIntMul.name().to_owned())
        .func_body();

    let a_ptr = module.locals.add(ValType::I32);
    let b_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Locals
    let pointer = module.locals.add(ValType::I32);
    let a = module.locals.add(ValType::I64);
    let b = module.locals.add(ValType::I64);
    // The row we are currently processing
    let a_offset = module.locals.add(ValType::I32);
    let b_offset = module.locals.add(ValType::I32);
    let carry_mul = module.locals.add(ValType::I64);
    let carry_sum = module.locals.add(ValType::I64);
    let partial_mul_res = module.locals.add(ValType::I64);
    let partial_sum_res = module.locals.add(ValType::I64);

    // Allocate memory for the result
    builder
        // Allocate memory for the result
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    builder
        .block(None, |outer_block| {
            let outer_block_id = outer_block.id();

            outer_block.loop_(None, |outer_loop| {
                let outer_loop_id = outer_loop.id();

                outer_loop
                    // If the offset is the same as the type_heap_size, we break the loop
                    .local_get(b_offset)
                    .local_get(type_heap_size)
                    .binop(BinaryOp::I32Eq)
                    .br_if(outer_block_id);

                // Set to zero partial results
                outer_loop
                    .i32_const(0)
                    .local_set(a_offset)
                    .i64_const(0)
                    .local_set(partial_sum_res)
                    .i64_const(0)
                    .local_set(partial_mul_res)
                    .i64_const(0)
                    .local_set(carry_sum)
                    .i64_const(0)
                    .local_set(carry_mul);

                // Load the first part
                outer_loop
                    // Read the second operand
                    .local_get(b_ptr)
                    .local_get(b_offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .unop(UnaryOp::I64ExtendUI32)
                    .local_set(b)
                    .block(None, |inner_block| {
                        let inner_block_id = inner_block.id();
                        // This loop is in charge of do the partial multiplications with a fixed part of b
                        // (b_n) and a moving part of a (a1, a2, ..., a_n)
                        inner_block.loop_(None, |loop_| {
                            let loop_id = loop_.id();

                            loop_
                                // If a_offset + b_offset = type_heap_size, means we processed the
                                // last chunk of digits
                                .local_get(a_offset)
                                .local_get(b_offset)
                                .binop(BinaryOp::I32Add)
                                .local_get(type_heap_size)
                                .binop(BinaryOp::I32Eq)
                                .if_else(
                                    None,
                                    |then| {
                                        // If there is carry in the multiplication, means we overflowed so we
                                        // trap
                                        // Otherwise we exit the inner loop and continue the
                                        // multiplication
                                        then.local_get(carry_mul)
                                            .i64_const(0)
                                            .binop(BinaryOp::I64Ne)
                                            .if_else(
                                                None,
                                                |then| {
                                                    then.unreachable();
                                                },
                                                |else_| {
                                                    else_.br(inner_block_id);
                                                },
                                            );
                                    },
                                    |_| {},
                                );

                            // Read the first operand
                            loop_
                                .local_get(a_ptr)
                                .local_get(a_offset)
                                .binop(BinaryOp::I32Add)
                                .load(
                                    compilation_ctx.memory_id,
                                    LoadKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                )
                                .unop(UnaryOp::I64ExtendUI32)
                                .local_tee(a)
                                .local_get(b)
                                // a_n * b_m + carry_mul
                                .binop(BinaryOp::I64Mul)
                                .local_get(carry_mul)
                                .binop(BinaryOp::I64Add)
                                .local_tee(partial_mul_res);

                            // We set the carry_mul as the higher 32 bits of the multiplication
                            // carry = (partial_mul_res >> 32)
                            loop_
                                .i64_const(32)
                                .binop(BinaryOp::I64ShrU)
                                .local_set(carry_mul)
                                .local_get(partial_mul_res)
                                // And we leave in the stack the lower 32 bits of the multiplication
                                // we will add this later
                                .i64_const(0x00000000FFFFFFFF)
                                .binop(BinaryOp::I64And);

                            // After calculating the partial multiplication and saving it in partial_mul_res we
                            // have the following:
                            // partial_mul_res = lower 32 bits of the multiplication
                            // carry_mul       = higher 32 bits of the multiplication
                            //
                            // Now we need to add that partial multilication result to the result. In order to
                            // do so, we need to load the corresponding part of the res we are processing (the
                            // a_offset) and add the partial_mul_res.
                            // After this addition we calculate if there is a carry for the ADDITION, and save
                            // it to use it in the addition of the next chunk.
                            //
                            // The chunk from the partial response to add is always shifted by the b offset
                            //    a4 a3 a2 a1
                            // x  b4 b3 b2 b1
                            //    -----------
                            //    c4 c3 c2 c1     b_offset = 0
                            // +  d3 d2 d1 0      b_offset = 4
                            //    e2 e1 0  0      b_offset = 8
                            //    f1 0  0  0      b_offset = 12

                            // First we load the part of res we need to add
                            loop_
                                .local_get(a_offset)
                                .local_get(b_offset)
                                .binop(BinaryOp::I32Add)
                                .local_get(pointer)
                                .binop(BinaryOp::I32Add)
                                .load(
                                    compilation_ctx.memory_id,
                                    LoadKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                )
                                .unop(UnaryOp::I64ExtendUI32)
                                // We add the lower 32 bits of the partial multiplication left in the stack
                                .binop(BinaryOp::I64Add)
                                // And add the carry of the previous addition, if any
                                .local_get(carry_sum)
                                .binop(BinaryOp::I64Add)
                                .local_set(partial_sum_res)
                                // After the additions we save it in res
                                .local_get(a_offset)
                                .local_get(b_offset)
                                .binop(BinaryOp::I32Add)
                                .local_get(pointer)
                                .binop(BinaryOp::I32Add)
                                // We use only the lower 32 bits of the partial sum res
                                .local_get(partial_sum_res)
                                .i64_const(0x00000000FFFFFFFF)
                                .binop(BinaryOp::I64And)
                                .unop(UnaryOp::I32WrapI64)
                                .store(
                                    compilation_ctx.memory_id,
                                    StoreKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                )
                                // Set the carry for the next sum
                                .local_get(partial_sum_res)
                                .i64_const(32)
                                .binop(BinaryOp::I64ShrU)
                                .local_set(carry_sum);

                            // a_offset += 4
                            loop_
                                .i32_const(4)
                                .local_get(a_offset)
                                .binop(BinaryOp::I32Add)
                                .local_set(a_offset)
                                .br(loop_id);
                        });
                    });

                // b_offset += 4
                outer_loop
                    .i32_const(4)
                    .local_get(b_offset)
                    .binop(BinaryOp::I32Add)
                    .local_set(b_offset)
                    .br(outer_loop_id);
            });
        })
        .local_get(pointer);
    function.finish(vec![a_ptr, b_ptr, type_heap_size], &mut module.funcs)
}

/// Multiply two u32 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// u32::MAX then the execution is aborted. To check the overflow:
/// Given n1 >= 0, n2 > 0
/// n1 * n2 > u32::MAX <=> n1 > u32::MAX / n2
///
/// So there will be an overflow if n2 != 0 && n1 > 32::MAX / n2
///
/// # Arguments:
///    - first u32 number to multiply
///    - second u32 number to multiply
/// # Returns:
///    - multiplication of the arguments
pub fn mul_u32(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::MulU32.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I32);
    let n2 = module.locals.add(ValType::I32);

    // Set the two opends to local variables and reinsert them to the stack to operate them
    builder
        //n2 != 0
        .local_get(n2)
        .i32_const(0)
        .binop(BinaryOp::I32Ne)
        .if_else(
            ValType::I32,
            |then| {
                // n1 > max / n2
                then.local_get(n1)
                    .i32_const(u32::MAX as i32)
                    .local_get(n2)
                    .binop(BinaryOp::I32DivU)
                    .binop(BinaryOp::I32GtU)
                    .if_else(
                        Some(ValType::I32),
                        |then| {
                            then.unreachable();
                        },
                        |else_| {
                            else_.local_get(n1).local_get(n2).binop(BinaryOp::I32Mul);
                        },
                    );
            },
            |else_| {
                else_.i32_const(0);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

/// Multiply two u64 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// u64::MAX then the execution is aborted. To check the overflow:
/// Given n1 >= 0, n2 > 0
/// n1 * n2 > u64::MAX <=> n1 > u64::MAX / n2
///
/// So there will be an overflow if n2 != 0 && n1 > u64::MAX / n2
///
/// # Arguments:
///    - first u64 number to multiply
///    - second u64 number to multiply
/// # Returns:
///    - multiplication of the arguments
pub fn mul_u64(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
        &[ValType::I64],
    );
    let mut builder = function
        .name(RuntimeFunction::MulU64.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);

    // Set the two opends to local variables and reinsert them to the stack to operate them
    builder
        // n2 != 0
        .local_get(n2)
        .i64_const(0)
        .binop(BinaryOp::I64Ne)
        .if_else(
            ValType::I64,
            |then| {
                // n1 > max / n2
                then.local_get(n1)
                    .i64_const(u64::MAX as i64)
                    .local_get(n2)
                    .binop(BinaryOp::I64DivU)
                    .binop(BinaryOp::I64GtU)
                    .if_else(
                        Some(ValType::I64),
                        |then| {
                            then.unreachable();
                        },
                        |else_| {
                            else_.local_get(n1).local_get(n2).binop(BinaryOp::I64Mul);
                        },
                    );
            },
            |else_| {
                else_.i64_const(0);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use alloy_primitives::U256;
    use rstest::rstest;
    use walrus::FunctionBuilder;

    use super::*;

    #[rstest]
    #[case(2, 2, 4)]
    #[case(0, 2, 0)]
    #[case(2, 0, 0)]
    #[case(1, 1, 1)]
    #[case(5, 5, 25)]
    #[case(u64::MAX as u128, 2, u64::MAX as u128 * 2)]
    #[case(2, u64::MAX as u128, u64::MAX as u128 * 2)]
    #[case(2, u64::MAX as u128 + 1, (u64::MAX as u128 + 1) * 2)]
    #[case(u64::MAX as u128, u64::MAX as u128, u64::MAX as u128 * u64::MAX as u128)]
    #[case::t_2pow63_x_2pow63(
        9_223_372_036_854_775_808,
        9_223_372_036_854_775_808,
        85_070_591_730_234_615_865_843_651_857_942_052_864
    )]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u128::MAX, 2, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u128::MAX, 5, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u128::MAX, u64::MAX as u128, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u64::MAX as u128 * 2, u64::MAX as u128 * 2, 0)]
    fn test_heap_mul_u128(#[case] n1: u128, #[case] n2: u128, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE * 2));

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_add (n1_ptr, n2_ptr and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_mul(&mut raw_module, &compilation_ctx);
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();

        let mut buff = vec![0; TYPE_HEAP_SIZE as usize * 3];
        memory.read(&mut store, 0, &mut buff).unwrap();

        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

    #[rstest]
    #[case(U256::from(2), U256::from(2), U256::from(4))]
    #[case(U256::from(0), U256::from(2), U256::from(0))]
    #[case(U256::from(2), U256::from(0), U256::from(0))]
    #[case(U256::from(1), U256::from(1), U256::from(1))]
    #[case(U256::from(5), U256::from(5), U256::from(25))]
    #[case(U256::from(u64::MAX), U256::from(2), U256::from(u64::MAX as u128 * 2))]
    #[case(U256::from(2), U256::from(u64::MAX), U256::from(u64::MAX as u128 * 2))]
    #[case(
        U256::from(2),
        U256::from(u64::MAX as u128 + 1),
        U256::from((u64::MAX as u128 + 1) * 2)
    )]
    #[case(
        U256::from(u64::MAX),
        U256::from(u64::MAX),
        U256::from(u64::MAX as u128 * u64::MAX as u128)
    )]
    #[case::t_2pow63_x_2pow63(
        U256::from(9_223_372_036_854_775_808_u128),
        U256::from(9_223_372_036_854_775_808_u128),
        U256::from(85_070_591_730_234_615_865_843_651_857_942_052_864_u128)
    )]
    #[case(
        U256::from(u128::MAX),
        U256::from(2),
        U256::from(u128::MAX) * U256::from(2)
    )]
    #[case(
        U256::from(u128::MAX),
        U256::from(5),
        U256::from(u128::MAX) * U256::from(5)
    )]
    #[case(
        U256::from(u128::MAX),
        U256::from(u128::MAX),
        U256::from(u128::MAX) * U256::from(u128::MAX)
    )]
    #[case(
        U256::from(u64::MAX as u128 * 2),
        U256::from(u64::MAX as u128 * 2),
        U256::from(u64::MAX as u128 * 2) * U256::from(u64::MAX as u128 * 2),
    )]
    #[case(
        U256::from(2),
        U256::from(u128::MAX) + U256::from(1),
        (U256::from(u128::MAX) + U256::from(1)) * U256::from(2)
    )]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(U256::MAX, U256::from(2), U256::from(0))]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(U256::MAX, U256::from(5), U256::from(0))]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(U256::MAX, U256::MAX, U256::from(0))]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(
        U256::from(u128::MAX) * U256::from(2),
        U256::from(u128::MAX) * U256::from(2),
        U256::from(0),
    )]
    fn test_heap_mul_u256(#[case] n1: U256, #[case] n2: U256, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE * 2));

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_add (n1_ptr, n2_ptr and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_mul(&mut raw_module, &compilation_ctx);
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let data = [n1.to_le_bytes::<32>(), n2.to_le_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();

        let mut buff = vec![0; TYPE_HEAP_SIZE as usize * 3];
        memory.read(&mut store, 0, &mut buff).unwrap();

        assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
    }

    #[rstest]
    #[case(0, 1, 0)]
    #[case(1, 0, 0)]
    #[case(0, u32::MAX as i32, 0)]
    #[case(u32::MAX as i32, 0, 0)]
    #[case(1, u32::MAX as i32, u32::MAX as i32)]
    #[case(u16::MAX as i32, u16::MAX as i32, (u16::MAX as u32 * u16::MAX as u32) as i32)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u32::MAX as i32, 2, -1)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(2, u32::MAX as i32, -1)]
    fn test_add_u32(#[case] n1: i32, #[case] n2: i32, #[case] expected: i32) {
        let (mut raw_module, _, _) = build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_l = raw_module.locals.add(ValType::I32);
        let n2_l = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_add (n1_ptr, n2_ptr and size in heap)
        func_body.local_get(n1_l).local_get(n2_l);

        let add_u32_f = mul_u32(&mut raw_module);
        func_body.call(add_u32_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }

    #[rstest]
    #[rstest]
    #[case(0, 1, 0)]
    #[case(1, 0, 0)]
    #[case(0, u32::MAX as i64, 0)]
    #[case(u64::MAX as i64, 0, 0)]
    #[case(1, u64::MAX as i64, u64::MAX as i64)]
    #[case(u32::MAX as i64, u32::MAX as i64, (u32::MAX as u64 * u32::MAX as u64) as i64)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u64::MAX as i64, 2, -1)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(2, u64::MAX as i64, -1)]
    fn test_mul_u64(#[case] n1: i64, #[case] n2: i64, #[case] expected: i64) {
        let (mut raw_module, _, _) = build_module(None);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I64, ValType::I64],
            &[ValType::I64],
        );

        let n1_l = raw_module.locals.add(ValType::I64);
        let n2_l = raw_module.locals.add(ValType::I64);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_add (n1_ptr, n2_ptr and size in heap)
        func_body.local_get(n1_l).local_get(n2_l);

        let add_u64_f = mul_u64(&mut raw_module);
        func_body.call(add_u64_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i64 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }
}

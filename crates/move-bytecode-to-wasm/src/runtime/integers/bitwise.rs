use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// This function implements the shift left for u128 and u256
///
/// # Arguments:
///    - pointer to the number to shift
///    - shift amount (i32) max 127 and 255 for u128 and u256 respectively, aborts otherwise
///    - how many bytes the number occupies in heap
/// # Returns:
///    - pointer to the result
pub fn heap_int_shift_left(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    // Function arguments
    let n_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);
    let shift_amount = module.locals.add(ValType::I32);

    let mut builder = function
        .name(RuntimeFunction::HeapIntShiftLeft.name().to_owned())
        .func_body();

    let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
    // Max value for the shift amount should be 127 for u128 and 255 for u256
    builder
        .local_get(shift_amount)
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Mul)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .call(check_overflow_f)
        .drop();

    // Locals
    let pointer = module.locals.add(ValType::I32);
    let word_shift = module.locals.add(ValType::I32);
    let bit_shift = module.locals.add(ValType::I32);
    let total_words = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let j = module.locals.add(ValType::I32);

    // Allocate memory for the result
    builder
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    builder
        .local_get(shift_amount)
        .i32_const(64)
        .binop(BinaryOp::I32DivU)
        .local_set(word_shift);

    builder
        .local_get(shift_amount)
        .i32_const(64)
        .binop(BinaryOp::I32RemU)
        .local_set(bit_shift);

    builder
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32DivU)
        .local_tee(total_words)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .local_set(i);

    // Pseudo code for reference:
    // N = words.length
    // result = array of N u64s initialized to 0

    // word_shift = shift / 64
    // bit_shift = shift % 64

    // for i in (N - 1) down to 0:
    //     j = i + word_shift
    //     if j < N:
    //         result[j] |= words[i] << bit_shift
    //     if bit_shift > 0 and j + 1 < N:
    //         result[j + 1] |= words[i] >> (64 - bit_shift)

    builder.loop_(None, |loop_| {
        let loop_id = loop_.id();

        loop_
            .local_get(i)
            .local_get(word_shift)
            .binop(BinaryOp::I32Add)
            .local_set(j);

        loop_.block(None, |block| {
            let block_id = block.id();
            block
                .local_get(j)
                .local_get(total_words)
                .binop(BinaryOp::I32GeU)
                .br_if(block_id);

            // prepare pointer
            block
                .local_get(pointer)
                .local_get(j)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add);

            block
                .local_get(pointer)
                .local_get(j)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            block
                .local_get(n_ptr)
                .local_get(i)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_get(bit_shift)
                .unop(UnaryOp::I64ExtendUI32)
                .binop(BinaryOp::I64Shl);

            block.binop(BinaryOp::I64Or).store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        });

        loop_.block(None, |block| {
            let block_id = block.id();
            block
                .local_get(bit_shift)
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .local_get(j)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_get(total_words)
                .binop(BinaryOp::I32GeU)
                .binop(BinaryOp::I32Or)
                .br_if(block_id);

            // prepare pointer
            block
                .local_get(pointer)
                .local_get(j)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add);

            block
                .local_get(pointer)
                .local_get(j)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            block
                .local_get(n_ptr)
                .local_get(i)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .i32_const(64)
                .local_get(bit_shift)
                .binop(BinaryOp::I32Sub)
                .unop(UnaryOp::I64ExtendUI32)
                .binop(BinaryOp::I64ShrU);

            block.binop(BinaryOp::I64Or).store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        });

        loop_
            .local_get(i)
            .i32_const(0)
            .binop(BinaryOp::I32GtU)
            .if_else(
                None,
                |then| {
                    then.local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Sub)
                        .local_set(i)
                        .br(loop_id);
                },
                |_| {},
            );
    });

    // Return the address of the sum
    builder.local_get(pointer);

    function.finish(vec![n_ptr, shift_amount, type_heap_size], &mut module.funcs)
}

/// This function implements the shift right for u128 and u256
///
/// # Arguments:
///    - pointer to the number to shift
///    - shift amount (i32) max 127 and 255 for u128 and u256 respectively, aborts otherwise
///    - how many bytes the number occupies in heap
/// # Returns:
///    - pointer to the result
pub fn heap_int_shift_right(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    // Function arguments
    let n_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);
    let shift_amount = module.locals.add(ValType::I32);

    let mut builder = function
        .name(RuntimeFunction::HeapIntShiftRight.name().to_owned())
        .func_body();

    let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
    // Max value for the shift amount should be 127 for u128 and 255 for u256
    builder
        .local_get(shift_amount)
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Mul)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .call(check_overflow_f)
        .drop();

    // Locals
    let pointer = module.locals.add(ValType::I32);
    let word_shift = module.locals.add(ValType::I32);
    let bit_shift = module.locals.add(ValType::I32);
    let total_words = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let j = module.locals.add(ValType::I32);

    // Allocate memory for the result
    builder
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    builder
        .local_get(shift_amount)
        .i32_const(64)
        .binop(BinaryOp::I32DivU)
        .local_set(word_shift);

    builder
        .local_get(shift_amount)
        .i32_const(64)
        .binop(BinaryOp::I32RemU)
        .local_set(bit_shift);

    builder
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32DivU)
        .local_set(total_words);

    builder.i32_const(0).local_set(i);

    // Pseudo code for reference:
    // N = words.length
    // result = array of N u64s initialized to 0

    // word_shift = shift / 64
    // bit_shift = shift % 64

    // for i from 0 to N - 1:
    //     j = i + word_shift
    //     if j < N:
    //         result[i] |= words[j] >> bit_shift
    //     if bit_shift > 0 and j + 1 < N:
    //         result[i] |= words[j + 1] << (64 - bit_shift)

    builder.loop_(None, |loop_| {
        let loop_id = loop_.id();

        loop_
            .local_get(i)
            .local_get(word_shift)
            .binop(BinaryOp::I32Add)
            .local_set(j);

        loop_.block(None, |block| {
            let block_id = block.id();
            block
                .local_get(j)
                .local_get(total_words)
                .binop(BinaryOp::I32GeU)
                .br_if(block_id);

            // prepare pointer
            block
                .local_get(pointer)
                .local_get(i)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add);

            block
                .local_get(pointer)
                .local_get(i)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            block
                .local_get(n_ptr)
                .local_get(j)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_get(bit_shift)
                .unop(UnaryOp::I64ExtendUI32)
                .binop(BinaryOp::I64ShrU);

            block.binop(BinaryOp::I64Or).store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        });

        loop_.block(None, |block| {
            let block_id = block.id();
            block
                .local_get(bit_shift)
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .local_get(j)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_get(total_words)
                .binop(BinaryOp::I32GeU)
                .binop(BinaryOp::I32Or)
                .br_if(block_id);

            // prepare pointer
            block
                .local_get(pointer)
                .local_get(i)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add);

            block
                .local_get(pointer)
                .local_get(i)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            block
                .local_get(n_ptr)
                .local_get(j)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .i32_const(8)
                .binop(BinaryOp::I32Mul)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .i32_const(64)
                .local_get(bit_shift)
                .binop(BinaryOp::I32Sub)
                .unop(UnaryOp::I64ExtendUI32)
                .binop(BinaryOp::I64Shl);

            block.binop(BinaryOp::I64Or).store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        });

        loop_
            .local_get(i)
            .local_get(total_words)
            .i32_const(1)
            .binop(BinaryOp::I32Sub)
            .binop(BinaryOp::I32LtU)
            .if_else(
                None,
                |then| {
                    then.local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_set(i)
                        .br(loop_id);
                },
                |_| {},
            );
    });

    // Return the address of the sum
    builder.local_get(pointer);

    function.finish(vec![n_ptr, shift_amount, type_heap_size], &mut module.funcs)
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
    #[case(128128u128, 10, 128128u128 << 10)]
    #[case(128128u128, 110, 128128u128 << 110)]
    #[case(u128::MAX, 110, u128::MAX << 110)]
    #[case(u128::MAX, 127, u128::MAX << 127)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u128::MAX, 128, 0)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u128::MAX, 180, 0)]
    fn test_u128_shift_left(#[case] n: u128, #[case] shift_amount: i32, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let shift_amount_local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Number to shift pointer
        func_body.i32_const(0);
        // Shift left amount
        func_body.local_get(shift_amount_local);
        // Heap size
        func_body.i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_left_f = heap_int_shift_left(&mut raw_module, &compilation_ctx);
        // Shift left
        func_body.call(shift_left_f);

        let function = function_builder.finish(vec![shift_amount_local], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let data = [n.to_le_bytes()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

    #[rstest]
    #[case(U256::from(128128u128), 10, U256::from(128128u128 << 10))]
    #[case(U256::MAX, 50, U256::MAX << 50)]
    #[case(U256::MAX, 110, U256::MAX << 110)]
    #[case(U256::MAX, 160, U256::MAX << 160)]
    #[case(U256::MAX, 180, U256::MAX << 180)]
    #[case(U256::MAX, 255, U256::MAX << 255)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(U256::MAX, 256, U256::ZERO)]
    fn test_u256_shift_left(#[case] n: U256, #[case] shift_amount: i32, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let shift_amount_local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Number to shift pointer
        func_body.i32_const(0);
        // Shift left amount
        func_body.local_get(shift_amount_local);
        // Heap size
        func_body.i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_left_f = heap_int_shift_left(&mut raw_module, &compilation_ctx);
        // Shift left
        func_body.call(shift_left_f);

        let function = function_builder.finish(vec![shift_amount_local], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let data = [n.to_le_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
    }

    #[rstest]
    #[case(128128u128, 10, 128128u128 >> 10)]
    #[case(128128u128, 110, 128128u128 >> 110)]
    #[case(u128::MAX, 110, u128::MAX >> 110)]
    #[case(u128::MAX, 127, u128::MAX >> 127)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u128::MAX, 128, 0)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u128::MAX, 180, 0)]
    fn test_u128_shift_right(#[case] n: u128, #[case] shift_amount: i32, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let shift_amount_local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Number to shift pointer
        func_body.i32_const(0);
        // Shift left amount
        func_body.local_get(shift_amount_local);
        // Heap size
        func_body.i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_right_f = heap_int_shift_right(&mut raw_module, &compilation_ctx);
        // Shift right
        func_body.call(shift_right_f);

        let function = function_builder.finish(vec![shift_amount_local], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n.to_le_bytes()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

    #[rstest]
    #[case(U256::from(128128u128), 10, U256::from(128128u128 >> 10))]
    #[case(U256::MAX, 50, U256::MAX >> 50)]
    #[case(U256::MAX, 110, U256::MAX >> 110)]
    #[case(U256::MAX, 160, U256::MAX >> 160)]
    #[case(U256::MAX, 180, U256::MAX >> 180)]
    #[case(U256::MAX, 255, U256::MAX >> 255)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(U256::MAX, 256, U256::ZERO)]
    fn test_u256_shift_right(#[case] n: U256, #[case] shift_amount: i32, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let shift_amount_local = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Number to shift pointer
        func_body.i32_const(0);
        // Shift left amount
        func_body.local_get(shift_amount_local);
        // Heap size
        func_body.i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_right_f = heap_int_shift_right(&mut raw_module, &compilation_ctx);
        // Shift right
        func_body.call(shift_right_f);

        let function = function_builder.finish(vec![shift_amount_local], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let data = [n.to_le_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
    }
}

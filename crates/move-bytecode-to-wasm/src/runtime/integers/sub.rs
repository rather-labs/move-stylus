use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// This function implements the substraction with borrow check for heap integers (u128 and u256)
///
/// # Arguments:
///    - pointer to the first number
///    - pointer to the second argument
///    - pointer where the res is saved
///    - how many bytes the number occupies in heap
/// # Returns:
///    - pointer to the result
pub fn heap_integers_sub(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function
        .name(RuntimeFunction::HeapIntSub.name().to_owned())
        .func_body();

    // Function arguments
    let n1_ptr = module.locals.add(ValType::I32);
    let n2_ptr = module.locals.add(ValType::I32);
    let pointer = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Locals
    let offset = module.locals.add(ValType::I32);
    let borrow = module.locals.add(ValType::I64);
    let sum = module.locals.add(ValType::I64);
    let partial_sub = module.locals.add(ValType::I64);
    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);

    builder
        // Set borrow to 0
        .i64_const(0)
        .local_set(borrow)
        // Set offset to 0
        .i32_const(0)
        .local_set(offset);

    builder
        .block(None, |block| {
            let block_id = block.id();

            block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // Break the loop of we processed all the chunks
                loop_
                    .local_get(offset)
                    .local_get(type_heap_size)
                    .binop(BinaryOp::I32Eq)
                    .br_if(block_id);

                // Load n1
                loop_
                    .local_get(n1_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(n1);

                // Load n2
                loop_
                    .local_get(n2_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(n2);

                // partial_sub = n1 - borrow - n2 = n1 - (borrow + n2)
                loop_
                    .local_get(n1)
                    .local_get(borrow)
                    .binop(BinaryOp::I64Sub)
                    .local_get(n2)
                    .binop(BinaryOp::I64Sub)
                    .local_tee(partial_sub)
                    .local_set(partial_sub);

                // Save chunk of 64 bits
                loop_
                    .local_get(pointer)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .local_get(partial_sub)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // Calculate new borrow
                // If n1 - borrow < n2 == n1 < n2 + borrow => new borrow
                // We also need to check that n2 + borrow did not overflow: if that's the case then
                // there is a new borrow
                // For example:
                // n2      = 0xFFFFFFFFFFFFFFFF  (max u64)
                // borrow  = 0x1
                // sum     = n2 + borrow = 0     (wraps around)
                //
                // But n2 + borrow is the total substracted from n1, so, if the sum overflows,
                // means we need one bit more to represent the substraction, so, we borrow.
                //
                // So, to check if we borrow, we check that
                // (n1 < n2 + borrow) || (n2 + borrow < n2)
                loop_
                    // sum = n2 + borrow
                    .local_get(n2)
                    .local_get(borrow)
                    .binop(BinaryOp::I64Add)
                    .local_set(sum)
                    // n1 < n2 + borrow
                    .local_get(n1)
                    .local_get(sum)
                    .binop(BinaryOp::I64LtU)
                    // sum < n2
                    .local_get(sum)
                    .local_get(n2)
                    .binop(BinaryOp::I64LtU)
                    .binop(BinaryOp::I32Or)
                    .unop(UnaryOp::I64ExtendUI32)
                    .local_set(borrow);

                // offset += 8 and process the next part of the integer
                loop_
                    .local_get(offset)
                    .i32_const(8)
                    .binop(BinaryOp::I32Add)
                    .local_set(offset)
                    .br(loop_id);
            });
        })
        .local_get(borrow)
        .i64_const(1)
        .binop(BinaryOp::I64Eq)
        .if_else(
            ValType::I32,
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(pointer);
            },
        );

    function.finish(
        vec![n1_ptr, n2_ptr, pointer, type_heap_size],
        &mut module.funcs,
    )
}

/// Substracts two u8, u16 or u32 numbers.
///
/// # Arguments:
///    - first number to substract
///    - second number to substract
/// # Returns:
///    - substracted number
pub fn sub_u32(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::SubU32.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I32);
    let n2 = module.locals.add(ValType::I32);

    // If n1 < n2 means the substraction will underflow, so we trap, otherwise we return the
    // substraction
    builder
        .local_get(n1)
        .local_get(n2)
        .binop(BinaryOp::I32LtU)
        .if_else(
            ValType::I32,
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(n1).local_get(n2).binop(BinaryOp::I32Sub);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

/// Substracts two u64 numbers.
///
/// # Arguments:
///    - first number to substract
///    - second number to substract
/// # Returns:
///    - substracted number
pub fn sub_u64(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
        &[ValType::I64],
    );
    let mut builder = function
        .name(RuntimeFunction::SubU32.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);

    // If n1 < n2 means the substraction will underflow, so we trap, otherwise we return the
    // substraction
    builder
        .local_get(n1)
        .local_get(n2)
        .binop(BinaryOp::I64LtU)
        .if_else(
            ValType::I64,
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(n1).local_get(n2).binop(BinaryOp::I64Sub);
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
    #[case(2, 1, 1)]
    #[case(8589934590, 4294967295, 4294967295_u128)]
    #[case(8589934592, 4294967296, 4294967296_u128)]
    #[case(36893488147419103232, 18446744073709551616, 18446744073709551616_u128)]
    #[case(
        158456325028528675187087900670,
        79228162514264337593543950335,
        79228162514264337593543950335_u128
    )]
    #[case(
        158456325028528675187087900672,
        79228162514264337593543950336,
        79228162514264337593543950336_u128
    )]
    #[case(u128::MAX, 42, u128::MAX - 42)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(1, 2, 0)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(4294967296, 8589934592, 0)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(18446744073709551616, 36893488147419103232, 0)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(79228162514264337593543950336, 158456325028528675187087900672, 0)]
    #[case(36893488147419103230, 18446744073709551615, 18446744073709551615_u128)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(1, u128::MAX, 0)]
    fn test_heap_sub_u128(#[case] n1: u128, #[case] n2: u128, #[case] expected: u128) {
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

        // arguments for heap_integers_sub (n1_ptr, n2_ptr, where to store the result and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_sub(&mut raw_module, &compilation_ctx);
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

    #[rstest]
    #[case(U256::from(2), U256::from(1), U256::from(1))]
    #[case(
        U256::from(8589934590_u128),
        U256::from(4294967295_u128),
        U256::from(4294967295_u128)
    )]
    #[case(
        U256::from(8589934592_u128),
        U256::from(4294967296_u128),
        U256::from(4294967296_u128)
    )]
    #[case(
        U256::from(36893488147419103230_u128),
        U256::from(18446744073709551615_u128),
        U256::from(18446744073709551615_u128)
    )]
    #[case(
        U256::from(36893488147419103232_u128),
        U256::from(18446744073709551616_u128),
        U256::from(18446744073709551616_u128)
    )]
    #[case(
        U256::from(158456325028528675187087900670_u128),
        U256::from(79228162514264337593543950335_u128),
        U256::from(79228162514264337593543950335_u128)
    )]
    #[case(
        U256::from(158456325028528675187087900672_u128),
        U256::from(79228162514264337593543950336_u128),
        U256::from(79228162514264337593543950336_u128)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(U256::from(1), U256::from(2), U256::from(0))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        U256::from(4294967296_u128),
        U256::from(8589934592_u128),
        U256::from(0)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        U256::from(18446744073709551616_u128),
        U256::from(36893488147419103232_u128),
        U256::from(0)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(158456325028528675187087900672_u128),
        U256::from(0)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
        U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),
        U256::from(0)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(

        U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
        U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),
        U256::from(0)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
        U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),
        U256::from(0)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(U256::from(1), U256::from(u128::MAX), U256::from(0))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(U256::from(1), U256::MAX, U256::from(0))]
    fn test_heap_sub_u256(#[case] n1: U256, #[case] n2: U256, #[case] expected: U256) {
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

        // arguments for heap_integers_sub (n1_ptr, n2_ptr, where to store the result and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_sub(&mut raw_module, &compilation_ctx);
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1.to_le_bytes::<32>(), n2.to_le_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let pointer: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
    }

    #[rstest]
    #[case(84, 42, 42)]
    #[case(256, 1, 255)]
    #[case(510, 255, 255)]
    #[case(u16::MAX as i32 + 1, u16::MAX as i32,1)]
    #[case(131070, 65535, 65535)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(42, 84, 42)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(255, 256, 1)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(255, 510, 255)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u16::MAX as i32, u16::MAX as i32 + 1, 1)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(65535, 131070, 65535)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(1, u32::MAX as i32, -1)]
    fn test_sub_u32(#[case] n1: i32, #[case] n2: i32, #[case] expected: i32) {
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

        let add_u32_f = sub_u32(&mut raw_module);
        // Shift left
        func_body.call(add_u32_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(84, 42, 42)]
    #[case(256, 1, 255)]
    #[case(510, 255, 255)]
    #[case(u16::MAX as i64 + 1, u16::MAX as i64, 1)]
    #[case(8589934590, 4294967295, 4294967295)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(42, 84, 42)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(255, 256, 1)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(255, 510, 255)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u16::MAX as i64, u16::MAX as i64 + 1, 1)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(65535, 131070, 65535)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u32::MAX as i64, u32::MAX as i64 + 1, 1)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(4294967295, 8589934590, 4294967295)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(1, u64::MAX as i64, -1)]
    fn test_sub_u64(#[case] n1: i64, #[case] n2: i64, #[case] expected: i64) {
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

        let add_u64_f = sub_u64(&mut raw_module);
        // Shift left
        func_body.call(add_u64_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i64 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }
}

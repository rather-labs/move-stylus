use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// This function implements the addition with overflow check for heap integers (u128 and u256)
///
/// # Arguments:
///    - pointer to the first number
///    - pointer to the second argument
///    - pointer where the res is saved
///    - how many bytes the number occupies in heap
/// # Returns:
///    - pointer to the result
pub fn heap_integers_add(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function
        .name(RuntimeFunction::HeapIntSum.name().to_owned())
        .func_body();

    // Function arguments
    let n1_ptr = module.locals.add(ValType::I32);
    let n2_ptr = module.locals.add(ValType::I32);
    let pointer = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Locals
    let offset = module.locals.add(ValType::I32);
    let overflowed = module.locals.add(ValType::I32);
    let partial_sum = module.locals.add(ValType::I64);
    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);

    // Allocate memory for the result
    builder
        // Set the offset to 0
        .i32_const(0)
        .local_set(offset)
        // Set the overflowed to false
        .i32_const(0)
        .local_set(overflowed);

    builder
        .block(None, |block| {
            let block_id = block.id();
            block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // If we evaluated all the chunks we exit the loop
                loop_
                    .local_get(offset)
                    .local_get(type_heap_size)
                    .binop(BinaryOp::I32Eq)
                    .br_if(block_id);

                // Load a part of the first operand and save it in n1
                loop_
                    // Read the first operand
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
                    .local_tee(n1)
                    // Read the second operand
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
                    .local_tee(n2)
                    // We add the two loaded parts
                    .binop(BinaryOp::I64Add)
                    // And add the rest of the previous operation
                    // Here we use the fact that the rest is always 1 and that the overflowed flag
                    // is either 1 if there was an overflow or 0 if not. If there was an overflow
                    // we need to add 1 to the sum so, we re-use the variable
                    .local_get(overflowed)
                    .unop(UnaryOp::I64ExtendUI32)
                    .binop(BinaryOp::I64Add)
                    // Save the result to partial_sum
                    .local_set(partial_sum);

                // Save chunk of 64 bits
                loop_
                    .local_get(pointer)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .local_get(partial_sum)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // Check overflow
                loop_
                    // If either n1 and n2 is zero or rest is not zero then there can be an overflow
                    // (n1 != 0) && (n2 != 0) || (rest != 0)
                    // where rest = overflowed
                    .local_get(n1)
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .local_get(n2)
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .binop(BinaryOp::I32And)
                    .local_get(overflowed)
                    .unop(UnaryOp::I64ExtendUI32)
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .binop(BinaryOp::I32Or);

                // If partial sum is less or equal than any of the sumands then an overflow ocurred
                // (partial_sum <= n1) || (partial_sum <= n2)
                loop_
                    .local_get(partial_sum)
                    .local_get(n1)
                    .binop(BinaryOp::I64LeU)
                    .local_get(partial_sum)
                    .local_get(n2)
                    .binop(BinaryOp::I64LeU)
                    .binop(BinaryOp::I32Or)
                    // If the following condition is true, there was overflow
                    // ((n1 != 0) && (n2 != 0) || (rest != 0)) && ((partial_sum <= n1) || (partial_sum <= n2))
                    .binop(BinaryOp::I32And)
                    .local_set(overflowed);

                // offset += 8 and process the next part of the integer
                loop_
                    .local_get(offset)
                    .i32_const(8)
                    .binop(BinaryOp::I32Add)
                    .local_set(offset)
                    .br(loop_id);
            });
        })
        .local_get(overflowed)
        .i32_const(1)
        .binop(BinaryOp::I32Eq)
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

/// Adds two u32 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// 4_294_967_295 then the execution is aborted. To check the overflow we check that the result
/// is strictly greater than the two operands. Because we are using i32 integer, if the
/// addition overflow, WASM wraps around the result.
///
/// # Arguments:
///    - first u32 number to add
///    - second u32 number to add
/// # Returns:
///    - addition of the arguments
pub fn add_u32(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::AddU32.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I32);
    let n2 = module.locals.add(ValType::I32);
    let res = module.locals.add(ValType::I32);

    // Set the two opends to local variables and reinsert them to the stack to operate them
    builder.local_get(n1).local_get(n2).binop(BinaryOp::I32Add);

    // We check that the result is greater than the two operands. If this check fails means
    // WASM an overflow occured.
    // if (res < n1) || (res < n2)
    // then trap
    // else return res
    builder
        .local_tee(res)
        .local_get(n1)
        .binop(BinaryOp::I32LtU)
        .local_get(res)
        .local_get(n2)
        .binop(BinaryOp::I32LtU)
        .binop(BinaryOp::I32Or)
        .if_else(
            Some(ValType::I32),
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(res);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

/// Adds two u64 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// 18_446_744_073_709_551_615 then the execution is aborted. To check the overflow we check
/// that the result is strictly greater than the two operands. Because we are using i64
/// integer, if the addition overflow, WASM wraps around the result.
///
/// # Arguments:
///    - first u64 number to add
///    - second u64 number to add
/// # Returns:
///    - addition of the arguments
pub fn add_u64(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
        &[ValType::I64],
    );
    let mut builder = function
        .name(RuntimeFunction::AddU64.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);
    let res = module.locals.add(ValType::I64);

    // Add the u64 numbers and set the result
    builder
        .local_get(n1)
        .local_get(n2)
        .binop(BinaryOp::I64Add)
        .local_tee(res);

    // We check that the result is greater than the two operands. If this check fails means
    // WASM an overflow occured.
    // if (res < n1) || (res < n2)
    // then trap
    // else return res
    builder
        .local_get(n1)
        .binop(BinaryOp::I64LtU)
        .local_get(res)
        .local_get(n2)
        .binop(BinaryOp::I64LtU)
        .binop(BinaryOp::I32Or)
        .if_else(
            Some(ValType::I64),
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(res);
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
    #[case(1, 1, 2)]
    #[case(4294967295, 4294967295, 8589934590)]
    #[case(4294967296, 4294967296, 8589934592)]
    #[case(18446744073709551615, 18446744073709551615, 36893488147419103230)]
    #[case(18446744073709551616, 18446744073709551616, 36893488147419103232)]
    #[case(
        79228162514264337593543950335,
        79228162514264337593543950335,
        158456325028528675187087900670
    )]
    #[case(
        79228162514264337593543950336,
        79228162514264337593543950336,
        158456325028528675187087900672
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u128::MAX, 42, 0)]
    fn test_heap_add_u128(#[case] n1: u128, #[case] n2: u128, #[case] expected: u128) {
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

        // arguments for heap_integers_add (n1_ptr, n2_ptr, where to store the result and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_add(&mut raw_module, &compilation_ctx);
        // Shift left
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
        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

    #[rstest]
    #[case(U256::from(1), U256::from(1), U256::from(2))]
    #[case(
        U256::from(4294967295_u128),
        U256::from(4294967295_u128),
        U256::from(8589934590_u128)
    )]
    #[case(
        U256::from(4294967296_u128),
        U256::from(4294967296_u128),
        U256::from(8589934592_u128)
    )]
    #[case(
        U256::from(18446744073709551615_u128),
        U256::from(18446744073709551615_u128),
        U256::from(36893488147419103230_u128)
    )]
    #[case(
        U256::from(18446744073709551616_u128),
        U256::from(18446744073709551616_u128),
        U256::from(36893488147419103232_u128)
    )]
    #[case(
        U256::from(79228162514264337593543950335_u128),
        U256::from(79228162514264337593543950335_u128),
        U256::from(158456325028528675187087900670_u128)
    )]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(79228162514264337593543950336_u128),
        U256::from(158456325028528675187087900672_u128)
    )]
    #[case(
       U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
       U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
       U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),)
    ]
    #[case(
       U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
       U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
       U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),)
    ]
    #[case(
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
       U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),)
    ]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(U256::MAX, U256::from(42), U256::from(0))]
    fn test_heap_add_u256(#[case] n1: U256, #[case] n2: U256, #[case] expected: U256) {
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

        // arguments for heap_integers_add (n1_ptr, n2_ptr, where to store the result and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_add(&mut raw_module, &compilation_ctx);
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
        assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
    }

    #[rstest]
    #[case(42, 42, 84)]
    #[case(255, 1, 256)]
    #[case(255, 255, 510)]
    #[case(u16::MAX as i32, 1, u16::MAX as i32 + 1)]
    #[case(65535, 65535, 131070)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u32::MAX as i32, 1, -1)]
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

        let add_u32_f = add_u32(&mut raw_module);
        // Shift left
        func_body.call(add_u32_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(42, 42, 84)]
    #[case(255, 1, 256)]
    #[case(255, 255, 510)]
    #[case(u16::MAX as i64, 1, u16::MAX as i64 + 1)]
    #[case(65535, 65535, 131070)]
    #[case(u32::MAX as i64, 1, u32::MAX as i64 + 1)]
    #[case(4294967295, 4294967295, 8589934590)]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(u64::MAX as i64, 1, u64::MAX as i64 + 1)]
    fn test_add_u64(#[case] n1: i64, #[case] n2: i64, #[case] expected: i64) {
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

        let add_u64_f = add_u64(&mut raw_module);
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

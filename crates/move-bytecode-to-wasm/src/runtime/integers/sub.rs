use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::{
    CompilationContext, data::RuntimeErrorData, error::RuntimeError,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::RuntimeFunction;

/// This function implements the substraction with borrow check for heap integers (u128 and u256)
///
/// # WASM Function Arguments
/// * `a_ptr` (i32) - pointer to the first number
/// * `b_ptr` (i32) - pointer to the second argument
/// * `res_ptr` (i32) - pointer where the res is saved
/// * `size` (i32) - how many bytes the number occupies in heap
///
/// # WASM Function Returns
/// * pointer to the result
pub fn heap_integers_sub(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> FunctionId {
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
                then.return_error(
                    module,
                    compilation_ctx,
                    Some(ValType::I32),
                    runtime_error_data,
                    RuntimeError::Overflow,
                );
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
/// # WASM Function Arguments
/// * `a` (i32) - first number to substract
/// * `b` (i32) - second number to substract
///
/// # WASM Function Returns
/// * substracted number
pub fn sub_u32(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> FunctionId {
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
                then.return_error(
                    module,
                    compilation_ctx,
                    Some(ValType::I32),
                    runtime_error_data,
                    RuntimeError::Overflow,
                );
            },
            |else_| {
                else_.local_get(n1).local_get(n2).binop(BinaryOp::I32Sub);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

/// Substracts two u64 numbers.
///
/// # WASM Function Arguments
/// * `a` (i64) - first number to substract
/// * `b` (i64) - second number to substract
///
/// # WASM Function Returns
/// * substracted number
pub fn sub_u64(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
        &[ValType::I64],
    );
    let mut builder = function
        .name(RuntimeFunction::SubU64.name().to_owned())
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
                then.return_error(
                    module,
                    compilation_ctx,
                    Some(ValType::I64),
                    runtime_error_data,
                    RuntimeError::Overflow,
                );
            },
            |else_| {
                else_.local_get(n1).local_get(n2).binop(BinaryOp::I64Sub);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;

    use crate::data::DATA_ABORT_MESSAGE_PTR_OFFSET;
    use crate::test_compilation_context;
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use alloy_primitives::U256;
    use alloy_primitives::keccak256;
    use alloy_sol_types::{SolType, sol};
    use rstest::rstest;
    use walrus::{FunctionBuilder, ValType};

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
    #[case(36893488147419103230, 18446744073709551615, 18446744073709551615_u128)]
    fn test_heap_sub_u128(#[case] n1: u128, #[case] n2: u128, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut store, instance, entrypoint) = setup_heap_sub_test(
            n1.to_le_bytes().to_vec(),
            n2.to_le_bytes().to_vec(),
            TYPE_HEAP_SIZE,
        );
        let result: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

    #[rstest]
    #[case(1u128, 2u128, 16)]
    #[case(4294967296u128, 8589934592u128, 16)]
    #[case(18446744073709551616u128, 36893488147419103232u128, 16)]
    #[case(
        79228162514264337593543950336u128,
        158456325028528675187087900672u128,
        16
    )]
    #[case(1u128, u128::MAX, 16)]
    fn test_heap_sub_overflow(#[case] n1: u128, #[case] n2: u128, #[case] heap_size: i32) {
        let (mut store, instance, entrypoint) = setup_heap_sub_test(
            n1.to_le_bytes().to_vec(),
            n2.to_le_bytes().to_vec(),
            heap_size,
        );
        let _: i32 = entrypoint.call(&mut store, (0, heap_size)).unwrap();
        assert_overflow_error(store, &instance);
    }

    #[test]
    fn test_heap_sub_u128_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut store, instance, entrypoint) = setup_heap_sub_test(
            vec![0; TYPE_HEAP_SIZE as usize],
            vec![0; TYPE_HEAP_SIZE as usize],
            TYPE_HEAP_SIZE,
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type()
            .for_each(|&(a, b): &(u128, u128)| {
                let data = [a.to_le_bytes(), b.to_le_bytes()].concat();

                memory.write(&mut *store.borrow_mut(), 0, &data).unwrap();

                let overflowing_sub = a.overflowing_sub(b);
                let expected = overflowing_sub.0;
                let overflows = overflowing_sub.1;

                let result: Result<i32, _> = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), (0, TYPE_HEAP_SIZE));

                match result {
                    Ok(pointer) => {
                        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                        if !overflows {
                            memory
                                .read(
                                    &mut *store.0.borrow_mut(),
                                    pointer as usize,
                                    &mut result_memory_data,
                                )
                                .unwrap();

                            assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
                        } else {
                            assert_eq!(0xBADF00D, pointer);
                        }
                    }
                    Err(_) => {
                        // In case of overflow we expect a trap
                        assert!(a.checked_sub(b).is_none());
                    }
                }

                reset_memory.call(&mut *store.borrow_mut(), ()).unwrap();
            });
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
    fn test_heap_sub_u256(#[case] n1: U256, #[case] n2: U256, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) = setup_heap_sub_test(
            n1.to_le_bytes::<32>().to_vec(),
            n2.to_le_bytes::<32>().to_vec(),
            TYPE_HEAP_SIZE,
        );

        let result: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
    }

    #[rstest]
    #[case(U256::from(1), U256::from(2), 32)]
    #[case(U256::from(4294967296_u128), U256::from(8589934592_u128), 32)]
    #[case(
        U256::from(18446744073709551616_u128),
        U256::from(36893488147419103232_u128),
        32
    )]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(158456325028528675187087900672_u128),
        32
    )]
    #[case(
        U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
        U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),
        32
    )]
    #[case(
        U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
        U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),
        32
    )]
    #[case(
        U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
        U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),
        32
    )]
    #[case(U256::from(1), U256::from(u128::MAX), 32)]
    #[case(U256::from(1), U256::MAX, 32)]
    fn test_heap_sub_u256_overflow(#[case] n1: U256, #[case] n2: U256, #[case] heap_size: i32) {
        let (mut store, instance, entrypoint) = setup_heap_sub_test(
            n1.to_le_bytes::<32>().to_vec(),
            n2.to_le_bytes::<32>().to_vec(),
            heap_size,
        );
        let _: i32 = entrypoint.call(&mut store, (0, heap_size)).unwrap();
        assert_overflow_error(store, &instance);
    }

    #[test]
    fn test_heap_sub_u256_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) = setup_heap_sub_test(
            vec![0; TYPE_HEAP_SIZE as usize],
            vec![0; TYPE_HEAP_SIZE as usize],
            TYPE_HEAP_SIZE,
        );
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type()
            .for_each(|&(a, b): &([u8; 32], [u8; 32])| {
                let a = U256::from_be_bytes(a);
                let b = U256::from_be_bytes(b);

                let data = [a.to_le_bytes::<32>(), b.to_le_bytes::<32>()].concat();

                memory.write(&mut *store.borrow_mut(), 0, &data).unwrap();

                let overflowing_sub = a.overflowing_sub(b);
                let expected = overflowing_sub.0;
                let overflows = overflowing_sub.1;

                let result: Result<i32, _> = entrypoint
                    .0
                    .call(&mut *store.0.borrow_mut(), (0, TYPE_HEAP_SIZE));

                match result {
                    Ok(pointer) => {
                        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                        if !overflows {
                            memory
                                .read(
                                    &mut *store.0.borrow_mut(),
                                    pointer as usize,
                                    &mut result_memory_data,
                                )
                                .unwrap();

                            assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
                        } else {
                            assert_eq!(0xBADF00D, pointer);
                        }
                    }
                    Err(_) => {
                        // In case of overflow we expect a trap
                        assert!(a.checked_sub(b).is_none());
                    }
                }

                reset_memory.call(&mut *store.borrow_mut(), ()).unwrap();
            });
    }

    #[rstest]
    #[case(84, 42, 42)]
    #[case(256, 1, 255)]
    #[case(510, 255, 255)]
    #[case(u16::MAX as i32 + 1, u16::MAX as i32,1)]
    #[case(131070, 65535, 65535)]
    fn test_sub_u32(#[case] n1: i32, #[case] n2: i32, #[case] expected: i32) {
        let (mut store, _instance, entrypoint) =
            setup_stack_sub_test::<(i32, i32), i32>(sub_u32, ValType::I32);
        let result: i32 = entrypoint.call(&mut store, (n1, n2)).unwrap();
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(42, 84)]
    #[case(255, 256)]
    #[case(255, 510)]
    #[case(u16::MAX as i32, u16::MAX as i32 + 1)]
    #[case(65535, 131070)]
    #[case(1, u32::MAX as i32)]
    fn test_sub_u32_overflow(#[case] n1: i32, #[case] n2: i32) {
        let (mut store, instance, entrypoint) =
            setup_stack_sub_test::<(i32, i32), i32>(sub_u32, ValType::I32);
        let _: i32 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_overflow_error(store, &instance);
    }

    #[test]
    fn test_sub_u32_fuzz() {
        let (mut store, instance, entrypoint) =
            setup_stack_sub_test::<(u32, u32), u32>(sub_u32, ValType::I32);

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u32, u32)>()
            .cloned()
            .for_each(|(a, b): (u32, u32)| {
                let expected = a.wrapping_sub(b);
                let mut store = store.borrow_mut();
                let result: Result<u32, _> = entrypoint.0.call(&mut *store, (a, b));

                match result {
                    Ok(res) => {
                        if a.checked_sub(b).is_none() {
                            // Overflow case: function should return 0xBADF00D
                            assert_eq!(0xBADF00D, res);
                        } else {
                            // Normal case: function should return the expected result
                            assert_eq!(expected, res);
                        }
                    }
                    Err(_) => {
                        // Overflows are handled by runtime errors so they dont longer trap
                    }
                }

                reset_memory.call(&mut *store, ()).unwrap();
            });
    }

    #[rstest]
    #[case(84, 42, 42)]
    #[case(256, 1, 255)]
    #[case(510, 255, 255)]
    #[case(u16::MAX as i64 + 1, u16::MAX as i64, 1)]
    #[case(8589934590, 4294967295, 4294967295)]
    fn test_sub_u64(#[case] n1: i64, #[case] n2: i64, #[case] expected: i64) {
        let (mut store, _instance, entrypoint) =
            setup_stack_sub_test::<(i64, i64), i64>(sub_u64, ValType::I64);
        let result: i64 = entrypoint.call(&mut store, (n1, n2)).unwrap();
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(42, 84)]
    #[case(255, 256)]
    #[case(255, 510)]
    #[case(u16::MAX as i64, u16::MAX as i64 + 1)]
    #[case(65535, 131070)]
    #[case(u32::MAX as i64, u32::MAX as i64 + 1)]
    #[case(4294967295, 8589934590)]
    #[case(1, u64::MAX as i64)]
    fn test_sub_u64_overflow(#[case] n1: i64, #[case] n2: i64) {
        let (mut store, instance, entrypoint) =
            setup_stack_sub_test::<(i64, i64), i64>(sub_u64, ValType::I64);
        let _: i64 = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_overflow_error(store, &instance);
    }

    #[test]
    fn test_sub_u64_fuzz() {
        let (mut store, instance, entrypoint) =
            setup_stack_sub_test::<(u64, u64), u64>(sub_u64, ValType::I64);

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u64, u64)>()
            .cloned()
            .for_each(|(a, b): (u64, u64)| {
                let expected = a.wrapping_sub(b);
                let mut store = store.borrow_mut();

                let result: Result<u64, _> = entrypoint.0.call(&mut *store, (a, b));

                match result {
                    Ok(res) => {
                        if a.checked_sub(b).is_none() {
                            // Overflow case: function should return 0xBADF00D
                            assert_eq!(0xBADF00D, res);
                        } else {
                            // Normal case: function should return the expected result
                            assert_eq!(expected, res);
                        }
                    }
                    Err(_) => {
                        // Overflows are handled by runtime errors so they dont longer trap
                    }
                }

                reset_memory.call(&mut *store, ()).unwrap();
            });
    }

    // Sets up a test module for sub operations on heap integers (u128 and u256)
    // Returns the store, instance, and entrypoint for external calling
    fn setup_heap_sub_test(
        n1_bytes: Vec<u8>,
        n2_bytes: Vec<u8>,
        heap_size: i32,
    ) -> (
        wasmtime::Store<()>,
        wasmtime::Instance,
        wasmtime::TypedFunc<(i32, i32), i32>,
    ) {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(heap_size * 2));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();

        let heap_integers_sub_f =
            heap_integers_sub(&mut raw_module, &compilation_ctx, &mut runtime_error_data);

        func_body
            .i32_const(0)
            .i32_const(heap_size)
            .i32_const(0)
            .i32_const(heap_size)
            .call(heap_integers_sub_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1_bytes, n2_bytes].concat();
        let (_, instance, store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, "test_function", None);

        (store, instance, entrypoint)
    }

    /// Sets up a test module for sub operations on stack integers (u32/u64)
    /// Returns the store, instance, and entrypoint for external calling
    fn setup_stack_sub_test<T, R>(
        sub_fn: impl FnOnce(&mut Module, &CompilationContext, &mut RuntimeErrorData) -> FunctionId,
        val_type: ValType,
    ) -> (
        wasmtime::Store<()>,
        wasmtime::Instance,
        wasmtime::TypedFunc<T, R>,
    )
    where
        T: wasmtime::WasmParams,
        R: wasmtime::WasmResults,
    {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) = build_module(None);
        let mut builder =
            FunctionBuilder::new(&mut raw_module.types, &[val_type, val_type], &[val_type]);
        let (n1_l, n2_l) = (
            raw_module.locals.add(val_type),
            raw_module.locals.add(val_type),
        );

        let ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let sub_f = sub_fn(&mut raw_module, &ctx, &mut RuntimeErrorData::new());
        builder
            .func_body()
            .local_get(n1_l)
            .local_get(n2_l)
            .call(sub_f);

        let function = builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);
        (store, instance, entrypoint)
    }

    /// Helper to verify that an overflow error was correctly written to memory
    fn assert_overflow_error(mut store: wasmtime::Store<()>, instance: &wasmtime::Instance) {
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Read the error pointer from the data segment
        let mut error_ptr_bytes = vec![0; 4];
        memory
            .read(
                &mut store,
                DATA_ABORT_MESSAGE_PTR_OFFSET as usize,
                &mut error_ptr_bytes,
            )
            .unwrap();

        let error_ptr = i32::from_le_bytes(error_ptr_bytes.try_into().unwrap());

        // If the error pointer is 0, it means that no error occurred
        assert_ne!(error_ptr, 0);

        // Load the length
        let mut error_length_bytes = vec![0; 4];
        memory
            .read(&mut store, error_ptr as usize, &mut error_length_bytes)
            .unwrap();

        let error_length = i32::from_le_bytes(error_length_bytes.try_into().unwrap());

        let mut result_data = vec![0; error_length as usize];
        memory
            .read(&mut store, (error_ptr + 4) as usize, &mut result_data)
            .unwrap();

        let error_message = String::from_utf8_lossy(RuntimeError::Overflow.as_bytes());
        let expected = [
            keccak256(b"Error(string)")[..4].to_vec(),
            <sol!((string,))>::abi_encode_params(&(error_message,)),
        ]
        .concat();
        assert_eq!(result_data, expected);
    }
}

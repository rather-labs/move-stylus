use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::{
    CompilationContext, data::RuntimeErrorData, runtime::error::RuntimeFunctionError,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::RuntimeFunction;

/// This function implements the shift left for u128 and u256
///
/// # WASM Function Arguments:
/// * `ptr` (i32) - pointer to the number to shift
/// * `shift_amount` (i32) - shift amount max 127 and 255 for u128 and u256 respectively, aborts otherwise
/// * `size` (i32) - how many bytes the number occupies in heap
///
/// # WASM Function Returns:
/// * pointer to the result
pub fn heap_int_shift_left(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
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

    let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;

    // Max value for the shift amount should be 127 for u128 and 255 for u256
    builder
        .local_get(shift_amount)
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Mul)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .call_runtime_function(
            compilation_ctx,
            check_overflow_f,
            &RuntimeFunction::CheckOverflowU8U16,
        )
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

    Ok(function.finish(vec![n_ptr, shift_amount, type_heap_size], &mut module.funcs))
}

/// This function implements the shift right for u128 and u256
///
/// # WASM Function Arguments:
/// * `ptr` (i32) - pointer to the number to shift
/// * `shift_amount` (i32) - shift amount max 127 and 255 for u128 and u256 respectively, aborts otherwise
/// * `size` (i32) - how many bytes the number occupies in heap
///
/// # WASM Function Returns:
/// * pointer to the result
pub fn heap_int_shift_right(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
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

    let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;
    // Max value for the shift amount should be 127 for u128 and 255 for u256
    builder
        .local_get(shift_amount)
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Mul)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .call_runtime_function(
            compilation_ctx,
            check_overflow_f,
            &RuntimeFunction::CheckOverflowU8U16,
        )
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

    Ok(function.finish(vec![n_ptr, shift_amount, type_heap_size], &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;

    use crate::data::DATA_ABORT_MESSAGE_PTR_OFFSET;
    use crate::error::RuntimeError;
    use crate::test_compilation_context;
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use alloy_primitives::U256;
    use alloy_primitives::keccak256;
    use alloy_sol_types::{SolType, sol};
    use rstest::rstest;
    use walrus::{FunctionBuilder, ValType};

    use super::*;

    #[rstest]
    #[case(128128u128, 10, 128128u128 << 10)]
    #[case(128128u128, 110, 128128u128 << 110)]
    #[case(u128::MAX, 110, u128::MAX << 110)]
    #[case(u128::MAX, 127, u128::MAX << 127)]
    fn test_u128_shift_left(#[case] n: u128, #[case] shift_amount: i32, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes().to_vec(), TYPE_HEAP_SIZE, false);

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
    #[case(u128::MAX, 128)]
    #[case(u128::MAX, 180)]
    fn test_u128_shift_left_overflow(#[case] n: u128, #[case] shift_amount: i32) {
        const TYPE_HEAP_SIZE: i32 = 16;

        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes().to_vec(), TYPE_HEAP_SIZE, false);

        let _: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        assert_overflow_error(&mut store, &instance);
    }

    #[test]
    fn test_u128_shift_left_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(vec![0; TYPE_HEAP_SIZE as usize], TYPE_HEAP_SIZE, false);

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
            .for_each(|&(n, shift): &(u128, u8)| {
                let mut store = store.borrow_mut();

                let data = n.to_le_bytes();
                memory.write(&mut *store, 0, &data).unwrap();

                let result: Result<i32, _> =
                    entrypoint.call(&mut *store, (shift as i32, TYPE_HEAP_SIZE));

                match result {
                    Ok(pointer) => {
                        if pointer != 0xBADF00D {
                            let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                            memory
                                .read(&mut *store, pointer as usize, &mut result_memory_data)
                                .unwrap();
                            let expected = n.checked_shl(shift as u32).unwrap_or(0);
                            assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
                        }
                    }
                    Err(_) => {
                        // In case of overflow, we expect a trap
                        let expected = n.checked_shl(shift as u32);
                        assert!(expected.is_none());
                    }
                }
                reset_memory.call(&mut *store, ()).unwrap();
            });
    }

    #[rstest]
    #[case(U256::from(128128u128), 10, U256::from(128128u128 << 10))]
    #[case(U256::MAX, 50, U256::MAX << 50)]
    #[case(U256::MAX, 110, U256::MAX << 110)]
    #[case(U256::MAX, 160, U256::MAX << 160)]
    #[case(U256::MAX, 180, U256::MAX << 180)]
    #[case(U256::MAX, 255, U256::MAX << 255)]
    fn test_u256_shift_left(#[case] n: U256, #[case] shift_amount: i32, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes::<32>().to_vec(), TYPE_HEAP_SIZE, false);

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
    #[case(U256::MAX, 256)]
    fn test_u256_shift_left_overflow(#[case] n: U256, #[case] shift_amount: i32) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes::<32>().to_vec(), TYPE_HEAP_SIZE, false);

        let _: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        assert_overflow_error(&mut store, &instance);
    }

    #[test]
    fn test_u256_shift_left_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(vec![0; TYPE_HEAP_SIZE as usize], TYPE_HEAP_SIZE, false);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!().with_type().for_each(
            |&(n, shift): &([u8; TYPE_HEAP_SIZE as usize], u16)| {
                let mut store = store.borrow_mut();

                memory.write(&mut *store, 0, &n).unwrap();

                let n = U256::from_le_bytes::<32>(n);

                let result: Result<i32, _> =
                    entrypoint.call(&mut *store, (shift as i32, TYPE_HEAP_SIZE));

                match result {
                    Ok(pointer) => {
                        if pointer != 0xBADF00D {
                            let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                            memory
                                .read(&mut *store, pointer as usize, &mut result_memory_data)
                                .unwrap();
                            let expected = n << shift as usize;
                            assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
                        } else {
                            // Data is empty because we are reseting memory, should we not reset it?
                            // assert_overflow_error(&mut *store, &instance);
                        }
                    }
                    Err(_) => {
                        // In case of overflow, we expect a trap
                        let expected = n.checked_shl(shift as usize);
                        assert!(expected.is_none());
                    }
                }
                reset_memory.call(&mut *store, ()).unwrap();
            },
        );
    }

    #[rstest]
    #[case(128128u128, 10, 128128u128 >> 10)]
    #[case(128128u128, 110, 128128u128 >> 110)]
    #[case(u128::MAX, 110, u128::MAX >> 110)]
    #[case(u128::MAX, 127, u128::MAX >> 127)]
    fn test_u128_shift_right(#[case] n: u128, #[case] shift_amount: i32, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;

        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes().to_vec(), TYPE_HEAP_SIZE, true);

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
    #[case(u128::MAX, 128)]
    #[case(u128::MAX, 180)]
    fn test_u128_shift_right_overflow(#[case] n: u128, #[case] shift_amount: i32) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes().to_vec(), TYPE_HEAP_SIZE, false);

        let _: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        assert_overflow_error(&mut store, &instance);
    }

    #[test]
    fn test_u128_shift_right_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(vec![0; TYPE_HEAP_SIZE as usize], TYPE_HEAP_SIZE, true);

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
            .for_each(|&(n, shift): &(u128, u8)| {
                let mut store = store.borrow_mut();

                let data = n.to_le_bytes();
                memory.write(&mut *store, 0, &data).unwrap();

                let result: Result<i32, _> =
                    entrypoint.call(&mut *store, (shift as i32, TYPE_HEAP_SIZE));

                match result {
                    Ok(pointer) => {
                        if pointer != 0xBADF00D {
                            let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                            memory
                                .read(&mut *store, pointer as usize, &mut result_memory_data)
                                .unwrap();
                            let expected = n.checked_shr(shift as u32).unwrap_or(0);
                            assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
                        }
                    }
                    Err(_) => {
                        // In case of overflow, we expect a trap
                        let expected = n.checked_shr(shift as u32);
                        assert!(expected.is_none());
                    }
                }
                reset_memory.call(&mut *store, ()).unwrap();
            });
    }

    #[rstest]
    #[case(U256::from(128128u128), 10, U256::from(128128u128 >> 10))]
    #[case(U256::MAX, 50, U256::MAX >> 50)]
    #[case(U256::MAX, 110, U256::MAX >> 110)]
    #[case(U256::MAX, 160, U256::MAX >> 160)]
    #[case(U256::MAX, 180, U256::MAX >> 180)]
    #[case(U256::MAX, 255, U256::MAX >> 255)]
    fn test_u256_shift_right(#[case] n: U256, #[case] shift_amount: i32, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes::<32>().to_vec(), TYPE_HEAP_SIZE, true);

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
    #[case(U256::MAX, 256)]
    fn test_u256_shift_right_overflow(#[case] n: U256, #[case] shift_amount: i32) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(n.to_le_bytes::<32>().to_vec(), TYPE_HEAP_SIZE, true);

        let _: i32 = entrypoint
            .call(&mut store, (shift_amount, TYPE_HEAP_SIZE))
            .unwrap();

        assert_overflow_error(&mut store, &instance);
    }

    #[test]
    fn test_u256_shift_right_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut store, instance, entrypoint) =
            setup_heap_shift_test(vec![0; TYPE_HEAP_SIZE as usize], TYPE_HEAP_SIZE, true);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!().with_type().for_each(
            |&(n, shift): &([u8; TYPE_HEAP_SIZE as usize], u16)| {
                let mut store = store.borrow_mut();

                memory.write(&mut *store, 0, &n).unwrap();

                let n = U256::from_le_bytes::<32>(n);

                let result: Result<i32, _> =
                    entrypoint.call(&mut *store, (shift as i32, TYPE_HEAP_SIZE));

                match result {
                    Ok(pointer) => {
                        if pointer != 0xBADF00D {
                            let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                            memory
                                .read(&mut *store, pointer as usize, &mut result_memory_data)
                                .unwrap();
                            let expected = n >> shift as usize;
                            assert_eq!(result_memory_data, expected.to_le_bytes::<32>().to_vec());
                        }
                    }
                    Err(_) => {
                        // In case of overflow, we expect a trap
                        let expected = n.checked_shr(shift as usize);
                        assert!(expected.is_none());
                    }
                }
                reset_memory.call(&mut *store, ()).unwrap();
            },
        );
    }

    fn setup_heap_shift_test(
        n_bytes: Vec<u8>,
        heap_size: i32,
        shift_right: bool,
    ) -> (
        wasmtime::Store<()>,
        wasmtime::Instance,
        wasmtime::TypedFunc<(i32, i32), i32>,
    ) {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(heap_size));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let shift_amount = raw_module.locals.add(ValType::I32);

        let shift_f = if shift_right {
            heap_int_shift_right(&mut raw_module, &compilation_ctx, &mut runtime_error_data)
                .unwrap()
        } else {
            heap_int_shift_left(&mut raw_module, &compilation_ctx, &mut runtime_error_data).unwrap()
        };

        function_builder
            .func_body()
            .i32_const(0)
            .local_get(shift_amount)
            .i32_const(heap_size)
            .call(shift_f);

        let function = function_builder.finish(vec![shift_amount], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n_bytes].concat();
        let (_, instance, store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, "test_function", None);

        (store, instance, entrypoint)
    }

    /// Helper to verify that an overflow error was correctly written to memory
    fn assert_overflow_error(store: &mut wasmtime::Store<()>, instance: &wasmtime::Instance) {
        let error_ptr = {
            let memory = instance.get_memory(&mut *store, "memory").unwrap();

            // Read the error pointer from the data segment
            let mut error_ptr_bytes = vec![0; 4];
            memory
                .read(
                    &mut *store,
                    DATA_ABORT_MESSAGE_PTR_OFFSET as usize,
                    &mut error_ptr_bytes,
                )
                .unwrap();

            i32::from_le_bytes(error_ptr_bytes.try_into().unwrap())
        };

        // If the error pointer is 0, it means that no error occurred
        assert_ne!(error_ptr, 0);

        let result_data = {
            let memory = instance.get_memory(&mut *store, "memory").unwrap();

            // Load the length
            let mut error_length_bytes = vec![0; 4];
            memory
                .read(&mut *store, error_ptr as usize, &mut error_length_bytes)
                .unwrap();

            let error_length = i32::from_le_bytes(error_length_bytes.try_into().unwrap());

            let mut result_data = vec![0; error_length as usize];
            memory
                .read(&mut *store, (error_ptr + 4) as usize, &mut result_data)
                .unwrap();

            result_data
        };

        let error_message = String::from_utf8_lossy(RuntimeError::Overflow.as_bytes());
        let expected = [
            keccak256(b"Error(string)")[..4].to_vec(),
            <sol!((string,))>::abi_encode_params(&(error_message,)),
        ]
        .concat();
        assert_eq!(result_data, expected);
    }
}

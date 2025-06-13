pub mod add;
pub mod bitwise;
pub mod div;
pub mod mul;
pub mod sub;

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, UnaryOp},
};

use crate::{CompilationContext, translation::intermediate_types::simple_integers::IU32};

use super::RuntimeFunction;

/// Checks if an u8 or u16 number overflowed.
///
/// If the number overflowed it traps, otherwise it leaves the number in the stack
///
/// # Arguments:
///    - number to be checked
///    - the max number admitted by the number to check's type
/// # Returns:
///    - the numeber passed as argument
pub fn check_overflow_u8_u16(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::CheckOverflowU8U16.name().to_owned())
        .func_body();

    let n = module.locals.add(ValType::I32);
    let max = module.locals.add(ValType::I32);

    builder
        .local_get(n)
        .local_get(max)
        .binop(BinaryOp::I32GtU)
        .if_else(
            Some(ValType::I32),
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(n);
            },
        );

    function.finish(vec![n, max], &mut module.funcs)
}

/// Downcast u64 number to u32
///
/// If the number is greater than u32::MAX it traps
///
/// # Arguments:
///    - u64 number
/// # Returns:
///    - u64 number casted as u32
pub fn downcast_u64_to_u32(module: &mut walrus::Module) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::DowncastU64ToU32.name().to_owned())
        .func_body();

    let n = module.locals.add(ValType::I64);

    builder
        .local_get(n)
        .i64_const(IU32::MAX_VALUE)
        .binop(BinaryOp::I64GtU)
        .if_else(
            Some(ValType::I32),
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(n).unop(UnaryOp::I32WrapI64);
            },
        );

    function.finish(vec![n], &mut module.funcs)
}

/// Downcast u128 or u256 number to u32
///
/// If the number is greater than u32::MAX it traps
///
/// # Arguments:
///    - pointer to the number to downcast
///    - the number of bytes that the number occupies in heap
/// # Returns:
///    - downcasted u128 or u256 number to u32
pub fn downcast_u128_u256_to_u32(
    module: &mut walrus::Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::DowncastU128U256ToU32.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let heap_size = module.locals.add(ValType::I32);
    let offset = module.locals.add(ValType::I32);

    builder.local_get(reader_pointer).load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Ensure the rest bytes are zero, otherwise would have overflowed
    builder.block(None, |inner_block| {
        let inner_block_id = inner_block.id();

        inner_block.i32_const(4).local_set(offset);

        inner_block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            loop_
                // reader_pointer += offset
                .local_get(reader_pointer)
                .local_get(offset)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .if_else(
                    None,
                    |then| {
                        // If we checked all the heap for zeroes we exit
                        then.local_get(heap_size)
                            .i32_const(4)
                            .binop(BinaryOp::I32Sub)
                            .local_get(offset)
                            .binop(BinaryOp::I32Eq)
                            .br_if(inner_block_id);

                        // Otherwise we add 4 to the offset and loop
                        then.i32_const(4)
                            .local_get(offset)
                            .binop(BinaryOp::I32Add)
                            .local_set(offset)
                            .br(loop_id);
                    },
                    |else_| {
                        else_.unreachable();
                    },
                );
        });
    });

    function.finish(vec![reader_pointer, heap_size], &mut module.funcs)
}

/// Downcast u128 or u256 number to u64
///
/// If the number is greater than u64::MAX it traps
///
/// # Arguments:
///    - pointer to the number to downcast
///    - the number of bytes that the number occupies in heap
/// # Returns:
///    - downcasted u128 or u256 number to u64
pub fn downcast_u128_u256_to_u64(
    module: &mut walrus::Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I64],
    );
    let mut builder = function
        .name(RuntimeFunction::DowncastU128U256ToU64.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);
    let heap_size = module.locals.add(ValType::I32);
    let offset = module.locals.add(ValType::I32);

    builder.local_get(reader_pointer).load(
        compilation_ctx.memory_id,
        LoadKind::I64 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Ensure the rest bytes are zero, otherwise would have overflowed
    builder.block(None, |inner_block| {
        let inner_block_id = inner_block.id();

        inner_block.i32_const(8).local_set(offset);

        inner_block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            loop_
                // reader_pointer += offset
                .local_get(reader_pointer)
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
                .i64_const(0)
                .binop(BinaryOp::I64Eq)
                .if_else(
                    None,
                    |then| {
                        // If we checked all the heap for zeroes we exit
                        then.local_get(heap_size)
                            .i32_const(8)
                            .binop(BinaryOp::I32Sub)
                            .local_get(offset)
                            .binop(BinaryOp::I32Eq)
                            .br_if(inner_block_id);

                        // Otherwise we add 4 to the offset and loop
                        then.i32_const(8)
                            .local_get(offset)
                            .binop(BinaryOp::I32Add)
                            .local_set(offset)
                            .br(loop_id);
                    },
                    |else_| {
                        else_.unreachable();
                    },
                );
        });
    });

    function.finish(vec![reader_pointer, heap_size], &mut module.funcs)
}

/// Function that checks if a big number is less than other.
///
/// This is done by comparing the most significant part of each number. For example, for two u256
/// numbers a and b where:
/// a = [a1, a2, a3, a4]
/// b = [b1, b2, b3, b4]
///
/// If      a1 < b1 -> true
/// Else if a1 > b1 -> false
/// Else check next
///
/// # Arguments
///    - pointer to a
///    - pointer to b
///    - how many double words (64bits) occupies in memory
/// # Returns:
///    - 1 if a < b, otherwise 0
pub fn check_if_a_less_than_b(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    // Function arguments
    let a_ptr = module.locals.add(ValType::I32);
    let b_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Local variables
    let a = module.locals.add(ValType::I64);
    let b = module.locals.add(ValType::I64);
    let res = module.locals.add(ValType::I32);
    let offset = module.locals.add(ValType::I32);

    let mut builder = function
        .name(RuntimeFunction::LessThan.name().to_owned())
        .func_body();

    builder
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Sub)
        .local_set(offset);

    builder
        .block(None, |block| {
            let block_id = block.id();

            block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // If we processed the chunks we exit the loop
                loop_
                    .local_get(offset)
                    .i32_const(0)
                    .binop(BinaryOp::I32LtS)
                    .if_else(
                        None,
                        |then| {
                            then.i32_const(0).local_set(res).br(block_id);
                        },
                        |_| {},
                    );

                // Load a chunk of a
                loop_
                    .local_get(a_ptr)
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
                    .local_tee(a);

                // Load a chunk of b
                loop_
                    .local_get(b_ptr)
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
                    .local_tee(b);

                // Make the comparisons
                // If a < b we break the loop
                loop_.binop(BinaryOp::I64LtU).local_tee(res).br_if(block_id);

                // Otherwise we check
                loop_
                    .local_get(a)
                    .local_get(b)
                    .binop(BinaryOp::I64Eq)
                    .if_else(
                        None,
                        // If a == b then we process the next chunk
                        |then| {
                            // offset -= 8
                            then.local_get(offset)
                                .i32_const(8)
                                .binop(BinaryOp::I32Sub)
                                .local_set(offset)
                                .br(loop_id);
                        },
                        // Otherwise means a > b, so we return false
                        |else_| {
                            else_.i32_const(0).return_();
                        },
                    );
            });
        })
        .local_get(res);

    function.finish(vec![a_ptr, b_ptr, type_heap_size], &mut module.funcs)
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
    #[case(1, 1, 0)]
    #[case(2, 1, 0)]
    #[case(0, 2, 1)]
    #[case(4294967295, 4294967295, 0)]
    #[case(4294967296, 4294967296, 0)]
    #[case(4294967295, 4294967296, 1)]
    #[case(4294967296, 4294967295, 0)]
    #[case(18446744073709551615, 18446744073709551615, 0)]
    #[case(18446744073709551616, 18446744073709551615, 0)]
    #[case(18446744073709551615, 18446744073709551616, 1)]
    #[case(18446744073709551616, 18446744073709551616, 0)]
    #[case(79228162514264337593543950335, 79228162514264337593543950335, 0)]
    #[case(79228162514264337593543950336, 79228162514264337593543950335, 0)]
    #[case(79228162514264337593543950335, 79228162514264337593543950336, 1)]
    #[case(79228162514264337593543950336, 79228162514264337593543950336, 0)]
    #[case(u128::MAX, 42, 0)]
    #[case(42, u128::MAX, 1)]
    fn test_a_less_than_b_u128(#[case] n1: u128, #[case] n2: u128, #[case] expected: i32) {
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
        let heap_integers_add_f = check_if_a_less_than_b(&mut raw_module, &compilation_ctx);
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(U256::from(1), U256::from(1), 0)]
    #[case(U256::from(2), U256::from(1), 0)]
    #[case(U256::from(0), U256::from(2), 1)]
    #[case(U256::from(4294967295_u128), U256::from(4294967295_u128), 0)]
    #[case(U256::from(4294967296_u128), U256::from(4294967296_u128), 0)]
    #[case(U256::from(4294967295_u128), U256::from(4294967296_u128), 1)]
    #[case(U256::from(4294967296_u128), U256::from(4294967295_u128), 0)]
    #[case(
        U256::from(18446744073709551615_u128),
        U256::from(18446744073709551615_u128),
        0
    )]
    #[case(
        U256::from(18446744073709551616_u128),
        U256::from(18446744073709551615_u128),
        0
    )]
    #[case(
        U256::from(18446744073709551615_u128),
        U256::from(18446744073709551616_u128),
        1
    )]
    #[case(
        U256::from(18446744073709551616_u128),
        U256::from(18446744073709551616_u128),
        0
    )]
    #[case(
        U256::from(79228162514264337593543950335_u128),
        U256::from(79228162514264337593543950335_u128),
        0
    )]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(79228162514264337593543950335_u128),
        0
    )]
    #[case(
        U256::from(79228162514264337593543950335_u128),
        U256::from(79228162514264337593543950336_u128),
        1
    )]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(79228162514264337593543950336_u128),
        0
    )]
    #[case(U256::from(u128::MAX), U256::from(u128::MAX), 0)]
    #[case(U256::from(u128::MAX) + U256::from(1), U256::from(u128::MAX), 0)]
    #[case(U256::from(u128::MAX), U256::from(u128::MAX) + U256::from(1), 1)]
    #[case(
       U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
       U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
       0
    )]
    #[case(
       U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
       U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
       0
    )]
    #[case(
       U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
       U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
       1
    )]
    #[case(
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
       0
    )]
    #[case(
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512896", 10).unwrap(),
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
       0
    )]
    #[case(
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
       U256::from_str_radix("6277101735386680763835789423207666416102355444464034512896", 10).unwrap(),
       1
    )]
    #[case(U256::MAX, U256::from(42), 0)]
    #[case(U256::from(42), U256::MAX, 1)]
    fn test_a_less_than_b_u256(#[case] n1: U256, #[case] n2: U256, #[case] expected: i32) {
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
        let heap_integers_add_f = check_if_a_less_than_b(&mut raw_module, &compilation_ctx);
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1.to_le_bytes::<32>(), n2.to_le_bytes::<32>()].concat();
        let (_, _, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();
        assert_eq!(result, expected);
    }
}

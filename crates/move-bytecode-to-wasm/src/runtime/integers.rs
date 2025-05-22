use walrus::{
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
    FunctionBuilder, FunctionId, Module, ValType,
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// This function implements the addition with overflow check for heap integers (u128 and u256)
pub fn heap_integers_add(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function
        .name(RuntimeFunction::HeapIntSum.name().to_owned())
        .func_body();

    // Function arguments
    let n1_ptr = module.locals.add(ValType::I32);
    let n2_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Locals
    let pointer = module.locals.add(ValType::I32);
    let offset = module.locals.add(ValType::I32);
    let overflowed = module.locals.add(ValType::I32);
    let partial_sum = module.locals.add(ValType::I64);
    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);

    // Allocate memory for the result
    builder
        // Allocate memory for the result
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(pointer)
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

                // We check if we are adding the last chunks of the operands
                loop_
                    .local_get(offset)
                    .local_get(type_heap_size)
                    .i32_const(8)
                    .binop(BinaryOp::I32Sub)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        None,
                        |then| {
                            // If an overflow happened in the last chunk, means the whole number
                            // overflowed
                            then.local_get(overflowed).if_else(
                                None,
                                |then| {
                                    then.unreachable();
                                },
                                // Otherwise we finished the  addition
                                |else_| {
                                    else_.br(block_id);
                                },
                            );
                        },
                        // If we are not in the last chunk, we continue the iteration
                        |else_| {
                            // offset += 8 and process the next part of the integer
                            else_
                                .local_get(offset)
                                .i32_const(8)
                                .binop(BinaryOp::I32Add)
                                .local_set(offset)
                                .br(loop_id);
                        },
                    );
            });
        })
        // Return the address of the sum
        .local_get(pointer);

    function.finish(vec![n1_ptr, n2_ptr, type_heap_size], &mut module.funcs)
}

/// Adds two u32 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// 4_294_967_295 then the execution is aborted. To check the overflow we check that the result
/// is strictly greater than the two operands. Because we are using i32 integer, if the
/// addition overflow, WASM wraps around the result.
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
    // if (res > n1) && (res > n2)
    // then return res
    // else trap
    builder
        .local_tee(res)
        .local_get(n1)
        .binop(BinaryOp::I32GtU)
        .local_get(res)
        .local_get(n2)
        .binop(BinaryOp::I32GtU)
        .binop(BinaryOp::I32And)
        .if_else(
            Some(ValType::I32),
            |then| {
                then.local_get(res);
            },
            |else_| {
                else_.unreachable();
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

    // Add the u64 numbers ans set the result
    builder
        .local_get(n1)
        .local_get(n2)
        .binop(BinaryOp::I64Add)
        .local_tee(res);

    // We check that the result is greater than the two operands. If this check fails means
    // WASM an overflow occured.
    // if (res > n1) && (res > n2)
    // then return res
    // else trap
    builder
        .local_get(n1)
        .binop(BinaryOp::I64GtU)
        .local_get(res)
        .local_get(n2)
        .binop(BinaryOp::I64GtU)
        .binop(BinaryOp::I32And)
        .if_else(
            Some(ValType::I64),
            |then| {
                then.local_get(res);
            },
            |else_| {
                else_.unreachable();
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

/// Checks if an u8 or u16 number overflowed.
///
/// If the number overflowed it traps, otherwise it leaves the number in the stack
pub fn check_overflow_u8_u16(module: &mut Module) -> FunctionId {
    // the number to check and the max number admitted by the quantity of bits to check overflow
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

/// Downcast u128 or u256 numbert to u32
///
/// If the number is greater than u32::MAX it traps
pub fn downcast_u128_u256_to_u32(
    module: &mut walrus::Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    // first argument: pointer to the number
    // second argument: the number of bytes that the number occupies in heap
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

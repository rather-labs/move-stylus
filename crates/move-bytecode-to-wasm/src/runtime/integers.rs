pub mod add;
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

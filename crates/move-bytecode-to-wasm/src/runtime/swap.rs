use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// Adds a function that swaps the bytes of an i32 value
/// Useful for converting between Big-endian and Little-endian
///
/// The function will only be added if it doesn't exist yet in the module
pub fn swap_i32_bytes_function(module: &mut Module) -> FunctionId {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let input_param = module.locals.add(ValType::I32);

    // Move byte 0 -> 3
    function_body
        .local_get(input_param)
        .i32_const(24)
        .binop(BinaryOp::I32ShrU);

    // Mask
    function_body.i32_const(0x000000FF).binop(BinaryOp::I32And);

    // Move byte 1 -> 2
    function_body
        .local_get(input_param)
        .i32_const(8)
        .binop(BinaryOp::I32ShrU);

    // Mask
    function_body
        .i32_const(0x0000FF00)
        .binop(BinaryOp::I32And)
        .binop(BinaryOp::I32Or);

    // Move byte 2 -> 1
    function_body
        .local_get(input_param)
        .i32_const(8)
        .binop(BinaryOp::I32Shl);
    // Mask
    function_body
        .i32_const(0x00FF0000)
        .binop(BinaryOp::I32And)
        .binop(BinaryOp::I32Or);

    // Move byte 3 -> 0
    function_body
        .local_get(input_param)
        .i32_const(24)
        .binop(BinaryOp::I32Shl);

    // Mask
    function_body
        .i32_const(0xFF000000u32 as i32)
        .binop(BinaryOp::I32And)
        .binop(BinaryOp::I32Or);

    function_builder.name(RuntimeFunction::SwapI32Bytes.name().to_owned());
    function_builder.finish(vec![input_param], &mut module.funcs)
}

/// Adds a function that swaps the bytes of an i64 value
/// Useful for converting between Big-endian and Little-endian
///
/// The function will only be added if it doesn't exist yet in the module
pub fn swap_i64_bytes_function(
    module: &mut Module,
    swap_i32_bytes_function: FunctionId,
) -> FunctionId {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I64]);
    let mut function_body = function_builder.func_body();

    let input_param = module.locals.add(ValType::I64);
    let upper = module.locals.add(ValType::I32);

    // Get the upper 32 bits if the u64 and swap them
    function_body
        .local_get(input_param)
        .i64_const(32)
        .binop(BinaryOp::I64ShrU)
        .unop(UnaryOp::I32WrapI64)
        .call(swap_i32_bytes_function)
        .local_set(upper);

    // Get the lower 32 bits if the u64 and swap them
    function_body
        .local_get(input_param)
        .unop(UnaryOp::I32WrapI64)
        .call(swap_i32_bytes_function);

    function_body
        .unop(UnaryOp::I64ExtendUI32)
        .i64_const(32)
        .binop(BinaryOp::I64Shl);

    function_body
        .local_get(upper)
        .unop(UnaryOp::I64ExtendUI32)
        .binop(BinaryOp::I64Or);

    function_builder.name(RuntimeFunction::SwapI64Bytes.name().to_owned());
    function_builder.finish(vec![input_param], &mut module.funcs)
}

/// TODO: Description
///
/// Arguments
/// - ptr to the region
/// - how many bytes occupies (must be multiple of 8)
pub fn swap_memory_bytes_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );
    let mut function_body = function_builder.func_body();

    // Arguments
    let origin_ptr = module.locals.add(ValType::I32);
    let dest_ptr = module.locals.add(ValType::I32);
    let size = module.locals.add(ValType::I32);

    // Locals
    let tmp = module.locals.add(ValType::I64);
    let counter = module.locals.add(ValType::I32);

    let swap_64 = RuntimeFunction::SwapI64Bytes.get(module, None);

    function_body.i32_const(0).local_set(counter);

    function_body.block(None, |block| {
        let block_id = block.id();
        block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            // Exit loop if we finished processing
            loop_
                .local_get(counter)
                .local_get(size)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            // Load chunk from memory
            loop_
                .local_get(origin_ptr)
                .local_get(counter)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_tee(tmp);

            // Swap
            loop_.call(swap_64).local_set(tmp);

            // Save result
            loop_
                .local_get(dest_ptr)
                .local_get(counter)
                .binop(BinaryOp::I32Add)
                .local_get(tmp)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            loop_
                .local_get(counter)
                .i32_const(8)
                .binop(BinaryOp::I32Add)
                .local_set(counter)
                .br(loop_id);
        });
    });

    function_builder.name(RuntimeFunction::SwapMemoryBytes.name().to_owned());
    function_builder.finish(vec![origin_ptr, dest_ptr, size], &mut module.funcs)
}

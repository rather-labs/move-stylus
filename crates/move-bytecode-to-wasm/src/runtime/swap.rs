use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, UnaryOp},
};

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

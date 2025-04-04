use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

#[cfg(test)]
pub fn display_module(module: &mut Module) {
    let wat = wasmprinter::print_bytes(module.emit_wasm()).expect("Failed to generate WAT");
    // print with line breaks
    println!("{}", wat.replace("\\n", "\n"));
}

/// Adds a function that swaps the bytes of an i32 value
/// Useful for converting between Big-endian and Little-endian
///
/// The function will only be added if it doesn't exist yet in the module
pub fn add_swap_i32_bytes_function(module: &mut Module) -> FunctionId {
    let existing_function = module.funcs.by_name("swap_i32_bytes");
    if let Some(function) = existing_function {
        return function;
    }

    let input_param = module.locals.add(ValType::I32);

    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    // Move byte 0 -> 3
    function_body.local_get(input_param);
    function_body.i32_const(24);
    function_body.binop(BinaryOp::I32ShrU);
    // Mask
    function_body.i32_const(0x000000FF);
    function_body.binop(BinaryOp::I32And);

    // Move byte 1 -> 2
    function_body.local_get(input_param);
    function_body.i32_const(8);
    function_body.binop(BinaryOp::I32ShrU);
    // Mask
    function_body.i32_const(0x0000FF00);
    function_body.binop(BinaryOp::I32And);
    function_body.binop(BinaryOp::I32Or);

    // Move byte 2 -> 1
    function_body.local_get(input_param);
    function_body.i32_const(8);
    function_body.binop(BinaryOp::I32Shl);
    // Mask
    function_body.i32_const(0x00FF0000);
    function_body.binop(BinaryOp::I32And);
    function_body.binop(BinaryOp::I32Or);

    // Move byte 3 -> 0
    function_body.local_get(input_param);
    function_body.i32_const(24);
    function_body.binop(BinaryOp::I32Shl);
    // Mask
    function_body.i32_const(0xFF000000u32 as i32);
    function_body.binop(BinaryOp::I32And);
    function_body.binop(BinaryOp::I32Or);

    function_builder.name("swap_i32_bytes".to_string());
    function_builder.finish(vec![input_param], &mut module.funcs)
}

/// Adds a function that swaps the bytes of an i64 value
/// Useful for converting between Big-endian and Little-endian
///
/// The function will only be added if it doesn't exist yet in the module
pub fn add_swap_i64_bytes_function(module: &mut Module) -> FunctionId {
    let existing_function = module.funcs.by_name("swap_i64_bytes");
    if let Some(function) = existing_function {
        return function;
    }

    let input_param = module.locals.add(ValType::I64);
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I64]);
    let mut function_body = function_builder.func_body();

    // Byte 0 -> 7
    function_body.local_get(input_param);
    function_body.i64_const(56);
    function_body.binop(BinaryOp::I64ShrU);
    // Mask
    function_body.i64_const(0x00000000000000FF);
    function_body.binop(BinaryOp::I64And);

    // Byte 1 -> 6
    function_body.local_get(input_param);
    function_body.i64_const(40);
    function_body.binop(BinaryOp::I64ShrU);
    // Mask
    function_body.i64_const(0x000000000000FF00);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    // Byte 2 -> 5
    function_body.local_get(input_param);
    function_body.i64_const(24);
    function_body.binop(BinaryOp::I64ShrU);
    // Mask
    function_body.i64_const(0x0000000000FF0000);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    // Byte 3 -> 4
    function_body.local_get(input_param);
    function_body.i64_const(8);
    function_body.binop(BinaryOp::I64ShrU);
    // Mask
    function_body.i64_const(0x00000000FF000000);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    // Byte 4 -> 3
    function_body.local_get(input_param);
    function_body.i64_const(8);
    function_body.binop(BinaryOp::I64Shl);
    // Mask
    function_body.i64_const(0x000000FF00000000);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    // Byte 5 -> 2
    function_body.local_get(input_param);
    function_body.i64_const(24);
    function_body.binop(BinaryOp::I64Shl);
    // Mask
    function_body.i64_const(0x0000FF0000000000);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    // Byte 6 -> 1
    function_body.local_get(input_param);
    function_body.i64_const(40);
    function_body.binop(BinaryOp::I64Shl);
    // Mask
    function_body.i64_const(0x00FF000000000000);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    // Byte 7 -> 0
    function_body.local_get(input_param);
    function_body.i64_const(56);
    function_body.binop(BinaryOp::I64Shl);
    // Mask
    function_body.i64_const(0xFF00000000000000u64 as i64);
    function_body.binop(BinaryOp::I64And);
    function_body.binop(BinaryOp::I64Or);

    function_builder.name("swap_i64_bytes".to_string());
    function_builder.finish(vec![input_param], &mut module.funcs)
}

use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

use super::RuntimeFunction;

/// Multiply two u32 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// 4_294_967_295 then the execution is aborted. To check the overflow:
/// Given n1 >= 0, n2 > 0
/// n1 * n2 > max <=> n1 > max / n2
///
/// So there will be an overflow if n2 != 0 && n1 > max / n2 where max = 4_294_967_295
///
/// # Arguments:
///    - first u32 number to add
///    - second u32 number to add
/// # Returns:
///    - addition of the arguments
pub fn mul_u32(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::MulU32.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I32);
    let n2 = module.locals.add(ValType::I32);

    // Set the two opends to local variables and reinsert them to the stack to operate them
    builder
        //n2 != 0
        .local_get(n2)
        .i32_const(0)
        .binop(BinaryOp::I32Ne)
        .if_else(
            ValType::I32,
            |then| {
                // n1 > max / n2
                then.local_get(n1)
                    .i32_const(u32::MAX as i32)
                    .local_get(n2)
                    .binop(BinaryOp::I32DivU)
                    .binop(BinaryOp::I32GtU)
                    .if_else(
                        Some(ValType::I32),
                        |then| {
                            then.unreachable();
                        },
                        |else_| {
                            else_.local_get(n1).local_get(n2).binop(BinaryOp::I32Mul);
                        },
                    );
            },
            |else_| {
                else_.i32_const(0);
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
pub fn mul_u64(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
        &[ValType::I64],
    );
    let mut builder = function
        .name(RuntimeFunction::MulU64.name().to_owned())
        .func_body();

    let n1 = module.locals.add(ValType::I64);
    let n2 = module.locals.add(ValType::I64);

    // Set the two opends to local variables and reinsert them to the stack to operate them
    builder
        // n2 != 0
        .local_get(n2)
        .i64_const(0)
        .binop(BinaryOp::I64Ne)
        .if_else(
            ValType::I64,
            |then| {
                // n1 > max / n2
                then.local_get(n1)
                    .i64_const(u64::MAX as i64)
                    .local_get(n2)
                    .binop(BinaryOp::I64DivU)
                    .binop(BinaryOp::I64GtU)
                    .if_else(
                        Some(ValType::I64),
                        |then| {
                            then.unreachable();
                        },
                        |else_| {
                            else_.local_get(n1).local_get(n2).binop(BinaryOp::I64Mul);
                        },
                    );
            },
            |else_| {
                else_.i64_const(0);
            },
        );

    function.finish(vec![n1, n2], &mut module.funcs)
}

#[cfg(test)]
mod tests {
    use crate::runtime::test_tools::{build_module, setup_wasmtime_module};
    use rstest::rstest;
    use walrus::FunctionBuilder;

    use super::*;

    #[rstest]
    #[case(0, 1, 0)]
    #[case(1, 0, 0)]
    #[case(0, u32::MAX as i32, 0)]
    #[case(u32::MAX as i32, 0, 0)]
    #[case(1, u32::MAX as i32, u32::MAX as i32)]
    #[case(u16::MAX as i32, u16::MAX as i32, (u16::MAX as u32 * u16::MAX as u32) as i32)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u32::MAX as i32, 2, -1)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(2, u32::MAX as i32, -1)]
    fn test_add_u32(#[case] n1: i32, #[case] n2: i32, #[case] expected: i32) {
        let (mut raw_module, _, _) = build_module();

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

        let add_u32_f = mul_u32(&mut raw_module);
        // Shift left
        func_body.call(add_u32_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let (_, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function");

        let result = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }

    #[rstest]
    #[rstest]
    #[case(0, 1, 0)]
    #[case(1, 0, 0)]
    #[case(0, u32::MAX as i64, 0)]
    #[case(u64::MAX as i64, 0, 0)]
    #[case(1, u64::MAX as i64, u64::MAX as i64)]
    #[case(u32::MAX as i64, u32::MAX as i64, (u32::MAX as u64 * u32::MAX as u64) as i64)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u64::MAX as i64, 2, -1)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(2, u64::MAX as i64, -1)]
    fn test_mul_u64(#[case] n1: i64, #[case] n2: i64, #[case] expected: i64) {
        let (mut raw_module, _, _) = build_module();

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

        let add_u64_f = mul_u64(&mut raw_module);
        // Shift left
        func_body.call(add_u64_f);

        let function = function_builder.finish(vec![n1_l, n2_l], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let (_, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function");

        let result = entrypoint.call(&mut store, (n1, n2)).unwrap();

        assert_eq!(expected, result);
    }
}

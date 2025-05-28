use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

use super::RuntimeFunction;

/// We implement the long multiplication algorithm.
///
/// We use chunks of 32 bits to be able to have carry, because it can be greater than 1. The
/// general idea is to implement the elementary's school algorithm. Given two numbers of 128 bits,
/// we divide then in four chunks of 32 bits.
///    a4 a3 a2 a1
/// x  b4 b3 b2 b1
///
/// And we proceed with (numbers with ' are carry of the operantion). }
///
/// The first iteration is:
///
/// 1. a1 b1 = r1_1         = c1
/// 2. a2 b1 = r1_2 + r1_1' = c2
/// 3. a3 b1 = r1_3 + r1_2' = c3
/// 4. a4 b1 = r1_4 + r1_3' = c4 -> If there's carry in the last column, overflow!
///
/// The second iteration is:
///
/// 1. a1 b2 = r2_1         = d1
/// 2. a2 b2 = r2_2 + r2_1' = d2
/// 3. a3 b2 = r2_3 + r2_2' = d3 -> If there's carry in the third column, overflow!
///
/// and so on..
///
/// The result is then
///
///    a4 a3 a2 a1
/// x  b4 b3 b2 b1
///    -----------
///    c4 c3 c2 c1
/// +  d3 d2 d1 0
///    e2 e1 0  0
///    f1 0  0  0
///
/// This means that we can optimize the carry detection and the overall performance of the
/// algorithm
///
pub fn heap_integers_mul(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::HeapIntMul.name().to_owned())
        .func_body();

    let a_ptr = module.locals.add(ValType::I32);
    let b_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Locals
    let pointer = module.locals.add(ValType::I32);
    let a = module.locals.add(ValType::I64);
    let b = module.locals.add(ValType::I64);
    // The row we are currently processing
    let row = module.locals.add(ValType::I32);
    let a_offset = module.locals.add(ValType::I32);
    let b_offset = module.locals.add(ValType::I32);
    let carry_mul = module.locals.add(ValType::I64);
    let carry_sum = module.locals.add(ValType::I64);
    let partial_mul_res = module.locals.add(ValType::I64);
    let partial_sum_res = module.locals.add(ValType::I64);

    // Allocate memory for the result
    builder
        // Allocate memory for the result
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(pointer)
        // Set we are processing the first row
        .i32_const(0)
        .local_set(row)
        // Set to zero partial results
        .i64_const(0)
        .local_set(partial_sum_res)
        .i64_const(0)
        .local_set(partial_mul_res)
        .i64_const(0)
        .local_set(carry_sum)
        .i64_const(0)
        .local_set(carry_mul)
        .i32_const(0)
        .local_set(a_offset)
        .i32_const(0)
        .local_set(b_offset);

    // Load the first part
    builder
        // Read the second operand
        .local_get(b_ptr)
        .local_get(b_offset)
        .binop(BinaryOp::I32Add)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .unop(UnaryOp::I64ExtendUI32)
        .local_set(b)
        .block(None, |block| {
            let block_id = block.id();
            // This loop is in charge of do the partial multiplications with a fixed part of b
            // (b_n) and a moving part of a (a1, a2, ..., a_n)
            block.loop_(None, |loop_| {
                let loop_id = loop_.id();
                loop_
                    // If the offset is the same as the type_heap_size, we break the loop
                    .local_get(a_offset)
                    .local_get(type_heap_size)
                    .binop(BinaryOp::I32Eq)
                    .br_if(block_id);

                // Read the first operand
                loop_
                        .local_get(a_ptr)
                        .local_get(a_offset)
                        .binop(BinaryOp::I32Add)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .unop(UnaryOp::I64ExtendUI32)
                        .local_tee(a)
                        .local_get(b)
                        // a_n * b_m + carry_mul
                        .binop(BinaryOp::I64Mul)
                        .local_get(carry_mul)
                        .binop(BinaryOp::I64Add)
                        .local_tee(partial_mul_res)
                        // We set the carry as the higher 32 bits of the multiplication
                        // carry = (partial_mul_res >> 32)
                        .i64_const(32)
                        .binop(BinaryOp::I64ShrU)
                        .local_set(carry_mul)
                        .local_get(partial_mul_res)
                        // And we set the partial_mul_res as the lower 32 bits of the multiplication
                        .i64_const(0x00000000FFFFFFFF)
                        .binop(BinaryOp::I64And)
                        /*
                        // And save that part to the corresponding part in res
                        // First we load the part contained in res
                        .local_get(pointer)
                        .local_get(a_offset)
                        .binop(BinaryOp::I32Add)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .unop(UnaryOp::I64ExtendUI32)
                        .binop(BinaryOp::I64Add)
                        */
                        .local_get(carry_sum)
                        .binop(BinaryOp::I64Add)
                        .local_set(partial_sum_res)
                        // And save that part to the corresponding part in res
                        .local_get(pointer)
                        .local_get(a_offset)
                        .binop(BinaryOp::I32Add)
                        // We use only the lower 32 bits of the partial sum res
                        .local_get(partial_sum_res)
                        .i64_const(0x00000000FFFFFFFF)
                        .binop(BinaryOp::I64And)
                        .unop(UnaryOp::I32WrapI64)
                        .store(
                            compilation_ctx.memory_id,
                            StoreKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        // Set the carry for the next sum
                        .local_get(partial_sum_res)
                        .i64_const(32)
                        .binop(BinaryOp::I64ShrU)
                        .local_set(carry_sum)
                        // a_offset += 4
                        .i32_const(4)
                        .local_get(a_offset)
                        .binop(BinaryOp::I32Add)
                        .local_set(a_offset)
                        .br(loop_id)
                        // asd
                        ;
            });
        })
        .local_get(pointer);

    function.finish(vec![a_ptr, b_ptr, type_heap_size], &mut module.funcs)
}

/// Multiply two u32 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// u32::MAX then the execution is aborted. To check the overflow:
/// Given n1 >= 0, n2 > 0
/// n1 * n2 > u32::MAX <=> n1 > u32::MAX / n2
///
/// So there will be an overflow if n2 != 0 && n1 > 32::MAX / n2
///
/// # Arguments:
///    - first u32 number to multiply
///    - second u32 number to multiply
/// # Returns:
///    - multiplication of the arguments
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

/// Multiply two u64 numbers.
///
/// Along with the addition code to check overflow is added. If the result is greater than
/// u64::MAX then the execution is aborted. To check the overflow:
/// Given n1 >= 0, n2 > 0
/// n1 * n2 > u64::MAX <=> n1 > u64::MAX / n2
///
/// So there will be an overflow if n2 != 0 && n1 > u64::MAX / n2
///
/// # Arguments:
///    - first u64 number to multiply
///    - second u64 number to multiply
/// # Returns:
///    - multiplication of the arguments
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
    #[case(2, 2, 4)]
    #[case(1, 1, 1)]
    #[case(5, 5, 25)]
    #[case(u64::MAX as u128, 2, u64::MAX as u128 * 2)]
    #[case(u64::MAX as u128 + 1, 2, (u64::MAX as u128 + 1) * 2)]
    fn test_heap_mul_u128(#[case] n1: u128, #[case] n2: u128, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module();

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

        let heap_integers_add_f = heap_integers_mul(
            &mut raw_module,
            &CompilationContext {
                memory_id,
                allocator: allocator_func,
                functions_arguments: &[],
                functions_returns: &[],
                module_signatures: &[],
                constants: &[],
            },
        );
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        // display_module(&mut raw_module);

        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function");

        let pointer = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(&mut store, pointer as usize, &mut result_memory_data)
            .unwrap();

        println!("Result: {result_memory_data:?} from pointer: {pointer}");

        let mut buff = vec![0; TYPE_HEAP_SIZE as usize * 3];
        memory.read(&mut store, 0, &mut buff).unwrap();
        println!("resultant memory {buff:?}");

        assert_eq!(result_memory_data, expected.to_le_bytes().to_vec());
    }

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

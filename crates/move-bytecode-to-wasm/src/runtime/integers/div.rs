use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, runtime::RuntimeFunction};

// Auxiliary function names
const F_A_LESS_THAN_B: &str = "a_less_than_b";
const F_SHIFT_64BITS_RIGHT: &str = "shift_64bits_right";

/// Implements the restoring division algorithm for 128 ans 256 bit integers
///
/// We assume that the base we are using is 64.
///
/// Given a number of 256 bits, we can think of it as composed in 4 chunks of 64 bit numbers.
/// The algorithm goes as follows:
/// let
///    D = [D1, D2, D3, D4]     dividend
///    d = [d1, d2, d3, d4]     divisor
///
/// 1. Initialize quotient and remainder to 0
///    q = [0, 0, 0, 0]         quotient
///    r = [0, 0, 0, 0]         remainder
///
/// 2. Loop for the quantity of digits 0..4
///    a. Shift remainder by 1 digit (64 bits)
///    b. Set r[3] = D[i]
///    c. If divisor > remainder -> q[i] = 0
///       Otherwise substract divisor from remainder until remainder < divisor and add 1 to a
///       counter c for each substraction.
///       Store q[i] = c
///
/// 3. After the loop:
///    q = dividend / divisor
///    r = dividend % divisor
///
/// For example, using base 10, lets do 350 / 13:
///
/// q = [0, 0, 0]
/// r = [0, 0, 0]
/// D = [3, 5, 0]
/// d = [0, 1, 3]
///
/// Iteration 0:
/// a. r << 1                  -> r = [0, 0, 0]
/// b. r[3] = D[0]             -> r = [0, 0, 3]
/// c. 13 > 3 => q[0] = 0      -> q = [0, 0, 0]
///
/// Iteration 1:
/// a. r << 1                  -> r = [0, 3, 0]
/// b. r[3] = D[1]             -> r = [0, 3, 5]
/// c. 13 < 35
///     r -= d = 35 - 13 = 22 | c = 1
///     r -= d = 22 - 13 =  9 | c = 2
///     9 < 13 break
///                            -> r = [0, 0, 9]
///     q[1] = c => q[1] = 2   -> q = [0, 2, 0]
///
/// Iteration 2:
/// a. r << 1                  -> r = [0, 9, 0]
/// b. r[3] = D[2]             -> r = [0, 9, 0]
/// c. divisor < remainder - 13 < 90
///     r -= d = 90 - 13 = 77 | c = 1
///     r -= d = 77 - 13 = 66 | c = 2
///     ...
///     r -= d = 25 - 13 = 12 | c = 6
///     12 < 13 break
///                            -> r = [0, 1, 2]
///     q[2] = c => q[2] = 6   -> q = [0, 2, 6]
///
/// Checking D = q * d + r => 350 = 26 * 13 + 12
///
/// NOTE: In the implementation indexes and operations are complemented because we are working in
/// little endian. The description of the algorithm and the example are in big endian.
///
/// # Arguments
///    - pointer to the dividend
///    - pointer to the divisor
///    - how many bytes the number occupies in heap
/// # Returns:
///    - pointer to the quotient
///    - pointer to the remainder
pub fn heap_integers_div(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32, ValType::I32],
    );

    let print_i32 = module.imports.get_func("", "print_i32").unwrap();
    let print_separator = module.imports.get_func("", "print_separator").unwrap();
    let print_i64 = module.imports.get_func("", "print_i64").unwrap();
    let print_u128 = module.imports.get_func("", "print_u128").unwrap();

    let shift_64bits_right_f = shift_64bits_right(module, compilation_ctx);
    let check_if_a_less_than_b_f = check_if_a_less_than_b(module, compilation_ctx);
    let sub_f = RuntimeFunction::HeapIntSub.get(module, Some(compilation_ctx));

    // Function arguments
    let dividend_ptr = module.locals.add(ValType::I32);
    let divisor_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Locals
    let remainder_ptr = module.locals.add(ValType::I32);
    let quotient_ptr = module.locals.add(ValType::I32);

    let offset = module.locals.add(ValType::I32);
    let substraction_counter = module.locals.add(ValType::I64);

    let mut builder = function
        .name(RuntimeFunction::HeapIntDiv.name().to_owned())
        .func_body();

    // We initialize the offset to the most significant bit
    builder
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Sub)
        .local_set(offset);

    builder
        // Allocate space for the remainder
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(remainder_ptr)
        // Allocate space for the quotient
        .local_get(type_heap_size)
        .call(compilation_ctx.allocator)
        .local_set(quotient_ptr);

    builder
        .block(None, |block| {
            let block_id = block.id();

            block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                loop_.local_get(offset).call(print_i32);
                // If we evaluated all the chunks we exit the loop
                loop_
                    .local_get(offset)
                    .i32_const(0)
                    .binop(BinaryOp::I32LtS)
                    .br_if(block_id);

                loop_.i32_const(1).call(print_i32);
                // Shift the remainder by 1 digit
                loop_
                    .local_get(remainder_ptr)
                    .local_get(type_heap_size)
                    .call(shift_64bits_right_f);

                loop_.i32_const(2).call(print_i32);
                // Set r[0] = D[i]
                loop_
                    .local_get(remainder_ptr)
                    .local_get(dividend_ptr)
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
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                loop_.i32_const(3).call(print_i32);
                // If remainder < divisor -> q[0]
                // Otherwise we loop substraction until divisor < remainder
                loop_
                    .local_get(remainder_ptr)
                    .local_get(divisor_ptr)
                    .local_get(type_heap_size)
                    .call(check_if_a_less_than_b_f)
                    .if_else(
                        None,
                        // remainder < divisor => q[i] = 0
                        |then| {
                            then.i32_const(4).call(print_i32);
                            then.local_get(quotient_ptr)
                                .local_get(offset)
                                .binop(BinaryOp::I32Add)
                                .i64_const(0)
                                .store(
                                    compilation_ctx.memory_id,
                                    StoreKind::I64 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
                        },
                        // otherwise we perform remainder -= divisor until remainder < divisor and we
                        // count each substraction in c. When the loop is finished q[i] = c
                        |else_| {
                            else_.i32_const(5).call(print_i32);
                            // Set the substraction counter in 0
                            else_.i64_const(0).local_set(substraction_counter);

                            else_.loop_(None, |substraction_loop| {
                                let substraction_loop_id = substraction_loop.id();

                                substraction_loop
                                    .local_get(substraction_counter)
                                    .call(print_i64);

                                substraction_loop.local_get(remainder_ptr).call(print_u128);
                                substraction_loop.local_get(divisor_ptr).call(print_u128);
                                // remainder -= divisor
                                substraction_loop
                                    .local_get(remainder_ptr)
                                    .local_get(divisor_ptr)
                                    .local_get(type_heap_size)
                                    .call(sub_f)
                                    .local_set(remainder_ptr);
                                substraction_loop.local_get(remainder_ptr).call(print_u128);

                                substraction_loop
                                    .local_get(remainder_ptr)
                                    .local_get(divisor_ptr)
                                    .local_get(type_heap_size)
                                    .call(check_if_a_less_than_b_f)
                                    .if_else(
                                        None,
                                        |then| {
                                            then.i32_const(6).call(print_i32);
                                            // If remainder < divisor means we finished substracting,
                                            // we set q[i] = substraction_counter
                                            then.local_get(quotient_ptr)
                                                .local_get(offset)
                                                .binop(BinaryOp::I32Add)
                                                .local_get(substraction_counter)
                                                .store(
                                                    compilation_ctx.memory_id,
                                                    StoreKind::I64 { atomic: false },
                                                    MemArg {
                                                        align: 0,
                                                        offset: 0,
                                                    },
                                                );
                                        },
                                        |else_| {
                                            else_.i32_const(7).call(print_i32);
                                            // Otherwise we add 1 to the substraction_counter and loop
                                            // again
                                            else_
                                                .local_get(substraction_counter)
                                                .i64_const(1)
                                                .binop(BinaryOp::I64Add)
                                                .local_set(substraction_counter)
                                                .br(substraction_loop_id);
                                        },
                                    );
                            });
                        },
                    )
                    // We substract 8 from the offset and continue the
                    // outer loop
                    .local_get(offset)
                    .i32_const(8)
                    .binop(BinaryOp::I32Sub)
                    .local_set(offset)
                    .br(loop_id);
            });
        })
        .local_get(quotient_ptr)
        .local_get(remainder_ptr);

    function.finish(
        vec![dividend_ptr, divisor_ptr, type_heap_size],
        &mut module.funcs,
    )
}

/// Auxiliary function that shifts right by the base.
///
/// This is done by moving chunks of 64 bits to the right. For example, for u256:
/// let a = [1, 2, 3, 4]
///
/// 1. First shift  -> a = [0, 1, 2, 3]
/// 2. Second shift -> a = [0, 0, 1, 2]
/// 3. Third shift  -> a = [0, 0, 0, 1]
///
/// # Arguments
///    - pointer to the number to shift
///    - how many bytes the number occupies in heap
fn shift_64bits_right(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);

    // Function arguments
    let a_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Local variables
    let tmp = module.locals.add(ValType::I64);
    let ptr_offset = module.locals.add(ValType::I32);

    let mut builder = function.name(F_SHIFT_64BITS_RIGHT.to_owned()).func_body();

    // We set 0 in the first place and copy the first value to move to the tmp variable
    builder
        .local_get(a_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(tmp)
        .local_get(a_ptr)
        .i64_const(0)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // The chunk number to process is in the offset 8
    builder
        .local_get(a_ptr)
        .i32_const(8)
        .binop(BinaryOp::I32Add)
        .local_set(ptr_offset);

    builder.block(None, |block| {
        let block_id = block.id();

        block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            // First we get in the stack the
            loop_
                .local_get(ptr_offset)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                // Update the current position
                .local_get(ptr_offset)
                .local_get(tmp)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                // Save the replaced value in tmp
                .local_set(tmp)
                .local_get(ptr_offset)
                .i32_const(8)
                .binop(BinaryOp::I32Add)
                .local_tee(ptr_offset)
                // If ptr_offset - a_ptr = type_heap_size means that we processed all the chunks,
                // in that case we exit
                .local_get(a_ptr)
                .binop(BinaryOp::I32Sub)
                .local_get(type_heap_size)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id)
                .br(loop_id);
        });
    });

    function.finish(vec![a_ptr, type_heap_size], &mut module.funcs)
}

/// Auxiliary function that checks if a big number is less than other.
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
fn check_if_a_less_than_b(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
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

    let mut builder = function.name(F_A_LESS_THAN_B.to_owned()).func_body();

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
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use alloy_primitives::U256;
    use rstest::rstest;
    use walrus::FunctionBuilder;
    use wasmtime::Linker;

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
        use wasmtime::Engine;

        use crate::{
            test_tools::{get_linker_with_host_debug_functions, inject_host_debug_functions},
            utils::display_module,
        };

        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE * 2));

        inject_host_debug_functions(&mut raw_module);

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

        let heap_integers_add_f = check_if_a_less_than_b(
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

        let linker = get_linker_with_host_debug_functions();

        println!("a:{:?}\nb:{:?}", n1.to_le_bytes(), n2.to_le_bytes());
        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (_, _, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let result: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(1, u64::MAX as u128 + 1)]
    #[case(42, 42 << 64)]
    #[case(u8::MAX as u128, (u8::MAX as u128) << 64)]
    #[case(u16::MAX as u128, (u16::MAX as u128) << 64)]
    #[case(u32::MAX as u128, (u32::MAX as u128) << 64)]
    #[case(u64::MAX as u128, (u64::MAX as u128) << 64)]
    fn test_shift_64bits_right_u128(#[case] a: u128, #[case] expected: u128) {
        use wasmtime::Engine;

        use crate::{
            test_tools::{get_linker_with_host_debug_functions, inject_host_debug_functions},
            utils::display_module,
        };

        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        inject_host_debug_functions(&mut raw_module);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for shift_64bits_right (a_ptr, size in heap)
        func_body.i32_const(0).i32_const(TYPE_HEAP_SIZE);

        let shift_64bits_right_f = shift_64bits_right(
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
        func_body.call(shift_64bits_right_f);

        let function = function_builder.finish(vec![a_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        display_module(&mut raw_module);

        let linker = get_linker_with_host_debug_functions();

        println!("a: {a:?}");
        let data = a.to_le_bytes();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<i32, ()>(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        entrypoint.call(&mut store, 0).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let result = &memory.data(&mut store)[0..TYPE_HEAP_SIZE as usize];
        assert_eq!(result, expected.to_le_bytes());
    }

    #[rstest]
    #[case(350, 13, 26, 12)]
    fn test_div_mod_u128(
        #[case] n1: u128,
        #[case] n2: u128,
        #[case] quotient: u128,
        #[case] remainder: u128,
    ) {
        use wasmtime::Engine;

        use crate::{
            test_tools::{get_linker_with_host_debug_functions, inject_host_debug_functions},
            utils::display_module,
        };

        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE * 2));

        inject_host_debug_functions(&mut raw_module);

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32, ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_add (n1_ptr, n2_ptr and size in heap)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(TYPE_HEAP_SIZE);

        let heap_integers_add_f = heap_integers_div(
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

        display_module(&mut raw_module);

        let linker = get_linker_with_host_debug_functions();

        println!("a:{:?}\nb:{:?}", n1.to_le_bytes(), n2.to_le_bytes());
        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let (quotient_ptr, remainder_ptr): (i32, i32) =
            entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(
                &mut store,
                quotient_ptr as usize,
                &mut quotient_result_memory_data,
            )
            .unwrap();

        let mut remainder_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(
                &mut store,
                remainder_ptr as usize,
                &mut remainder_result_memory_data,
            )
            .unwrap();

        assert_eq!(quotient_result_memory_data, quotient.to_le_bytes());
        assert_eq!(remainder_result_memory_data, remainder.to_le_bytes());
    }
}

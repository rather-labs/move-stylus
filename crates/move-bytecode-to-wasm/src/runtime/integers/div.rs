use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, error::RuntimeFunctionError},
};

// Auxiliary function names
const F_SHIFT_64BITS_LEFT: &str = "shift_64bits_left";
const F_GET_BIT: &str = "get_bit";
const F_SET_BIT: &str = "set_bit";

/// Implements the long division algorithm for 128 and 256 bit integers.
///
/// A 256-bit number is treated as four 64-bit chunks:
///
///    D = [D1, D2, D3, D4]     Dividend
///    d = [d1, d2, d3, d4]     Divisor
///
/// ### Algorithm Steps:
///
/// 1. Initialize the quotient and remainder to zero:
///    q = [0, 0, 0, 0]         Quotient
///    r = [0, 0, 0, 0]         Remainder
///
/// 2. For each digit `i` from 0 to 3:
///    a. Left-shift the remainder by one chunk (64 bits).
///    b. Set `r[3] = D[i]`.
///    c. If the divisor is greater than the remainder, set `q[i] = 0`.
///    Otherwise, repeatedly subtract the divisor from the remainder until the remainder is
///    less than the divisor. Count how many subtractions were performed (`c`),
///    and set `q[i] = c`.
///
/// 3. After the loop:
///     - `q` holds the result of `dividend / divisor`.
///     - `r` holds the result of `dividend % divisor`.
///
/// ### Example (Base 10): Compute 350 ÷ 13
///
/// ```text
/// Initial state:
/// q = [0, 0, 0]
/// r = [0, 0, 0]
/// D = [3, 5, 0]   // 350
/// d = [0, 1, 3]   // 13
///
/// Iteration 0:
/// a. r << 1                  → r = [0, 0, 0]
/// b. r[2] = D[0]             → r = [0, 0, 3]
/// c. 13 > 3 → q[0] = 0       → q = [0, 0, 0]
///
/// Iteration 1:
/// a. r << 1                  → r = [0, 3, 0]
/// b. r[2] = D[1]             → r = [0, 3, 5]
/// c. 35 - 13 = 22            → c = 1
///    22 - 13 = 9             → c = 2
///    9 < 13 (stop)           → r = [0, 0, 9], q[1] = 2
///
/// Iteration 2:
/// a. r << 1                  → r = [0, 9, 0]
/// b. r[2] = D[2]             → r = [0, 9, 0] (no change)
/// c. 90 - 13 = 77            → c = 1
///    77 - 13 = 64            → c = 2
///    ...
///    25 - 13 = 12            → c = 6
///    12 < 13 (stop)          → r = [0, 1, 2], q[2] = 6
/// Final check: 26 * 13 + 12 = 350
/// ```
///
/// **Note:** In the implementation, indices and operations are reversed because we work in
/// little-endian format. This description and the example assume big-endian for clarity.
///
/// # Arguments
/// - Pointer to the dividend
/// - Pointer to the divisor
/// - Number of bits the values occupy in memory
/// - Whether return remainder or quotient. 1 for quotient, 0 for remainder.
///
/// # Returns
/// - Pointer to the result
pub fn heap_integers_div_mod(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let check_if_a_less_than_b_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;
    let sub_f = RuntimeFunction::HeapIntSub.get(module, Some(compilation_ctx))?;
    let left_shift_number = shift_1bit_left(module, compilation_ctx);
    let get_bit = get_bit(module, compilation_ctx);
    let set_bit = set_bit(module, compilation_ctx);

    // Function arguments
    let dividend_ptr = module.locals.add(ValType::I32);
    let divisor_ptr = module.locals.add(ValType::I32);
    let n = module.locals.add(ValType::I32);
    let quotient_or_reminder = module.locals.add(ValType::I32);

    // Locals
    let remainder_ptr = module.locals.add(ValType::I32);
    let quotient_ptr = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);

    // Number of bytes occupied by the numbers
    let n_bytes = module.locals.add(ValType::I32);

    // To check if divisor is 0
    let accumulator = module.locals.add(ValType::I64);

    let mut builder = function
        .name(RuntimeFunction::HeapIntDivMod.name().to_owned())
        .func_body();

    /*
    // Before anything we check if divisor is 0
    // TODO: replace with iszero runtime function
    builder.block(None, |block| {
        let block_id = block.id();
        block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            loop_
                .local_get(offset)
                .local_get(type_heap_size)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            loop_
                .local_get(divisor_ptr)
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
                .local_get(accumulator)
                .binop(BinaryOp::I64Or)
                .local_set(accumulator);

            // offset += 8
            loop_
                .i32_const(8)
                .local_get(offset)
                .binop(BinaryOp::I32Add)
                .local_set(offset)
                .br(loop_id);
        });
    });

    // If the accumulator == 0 means the divisor was 0. We divide by 0 to cause a runtime error
    // divided by 0
    builder
        .local_get(accumulator)
        .i64_const(0)
        .binop(BinaryOp::I64Eq)
        .if_else(
            None,
            |then| {
                then.i32_const(1)
                    .i32_const(0)
                    .binop(BinaryOp::I32DivU)
                    .drop();
            },
            |_| {},
        );

    // We initialize the offset to the most significant bit
    builder
        .local_get(type_heap_size)
        .i32_const(8)
        .binop(BinaryOp::I32Sub)
        .local_set(offset);

        */

    builder
        .local_get(n)
        .i32_const(8)
        .binop(BinaryOp::I32DivU)
        .local_set(n_bytes);

    builder
        // Allocate space for the remainder
        .local_get(n_bytes)
        .call(compilation_ctx.allocator)
        .local_set(remainder_ptr)
        // Allocate space for the quotient
        .local_get(n_bytes)
        .call(compilation_ctx.allocator)
        .local_set(quotient_ptr);

    // We loop from the most to the least significant bit of the numerator
    builder
        .local_get(n)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .local_set(i);

    builder.loop_(None, |loop_| {
        let loop_id = loop_.id();

        //
        // R := R << 1
        //
        loop_
            .local_get(remainder_ptr)
            //.local_get(n)
            .local_get(n_bytes)
            .call(left_shift_number);

        //
        // R(0) := N(i)
        //

        // Set the least significant bit of R equal to bit i of the numerator. The least
        // significant bit is R[0][7]. The last bit of the first byte, since we are in little
        // endian...

        loop_.local_get(remainder_ptr);

        // Fist get the first byte of the remainder
        loop_.local_get(remainder_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I64_8 {
                kind: walrus::ir::ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Get the bit of the dividen (N(i))
        loop_
            .local_get(dividend_ptr)
            .local_get(i)
            .local_get(n)
            .call(get_bit);

        // Merge it with the reminder byte
        loop_.binop(BinaryOp::I64Or);

        // Save it back to memory
        loop_.store(
            compilation_ctx.memory_id,
            StoreKind::I64_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        //
        // if R ≥ D then
        //

        loop_.block(None, |block_| {
            let block_id = block_.id();

            // If R < D we exit the block
            block_
                .local_get(remainder_ptr)
                .local_get(divisor_ptr)
                .local_get(n_bytes)
                .call(check_if_a_less_than_b_f)
                .br_if(block_id);

            // R := R − D
            block_
                .local_get(remainder_ptr)
                .local_get(divisor_ptr)
                .local_get(remainder_ptr)
                .local_get(n_bytes)
                .call(sub_f)
                .drop();

            // Q(i) := 1
            block_
                .local_get(quotient_ptr)
                .local_get(i)
                .local_get(n)
                .call(set_bit);
        });

        // i -= 1
        loop_
            .local_get(i)
            .i32_const(1)
            .binop(BinaryOp::I32Sub)
            .local_tee(i);

        loop_.i32_const(0).binop(BinaryOp::I32GeS).br_if(loop_id);
    });

    builder.local_get(quotient_or_reminder).if_else(
        ValType::I32,
        |then| {
            then.local_get(quotient_ptr);
        },
        |else_| {
            else_.local_get(remainder_ptr);
        },
    );

    Ok(function.finish(
        vec![dividend_ptr, divisor_ptr, n, quotient_or_reminder],
        &mut module.funcs,
    ))
}

/// ptr - number pointer
/// n - number of bytes the number occupies in memory
fn shift_1bit_left(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);

    let mut builder = function.name(F_SHIFT_64BITS_LEFT.to_owned()).func_body();

    // Function arguments
    let a_ptr = module.locals.add(ValType::I32);
    let n = module.locals.add(ValType::I32);

    let i = module.locals.add(ValType::I32);
    let addr = module.locals.add(ValType::I32);
    let word = module.locals.add(ValType::I64);
    let carry = module.locals.add(ValType::I64);
    let next = module.locals.add(ValType::I64);

    builder.i64_const(0).local_set(carry);
    builder.i32_const(0).local_set(i);

    builder.loop_(None, |loop_| {
        let loop_id = loop_.id();

        // addr of word i
        loop_
            .local_get(i)
            .i32_const(3)
            .binop(BinaryOp::I32Shl)
            .local_get(a_ptr)
            .binop(BinaryOp::I32Add)
            .local_tee(addr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(word);

        // Capture MSB before changing the word (next carry)
        loop_
            .local_get(word)
            .i64_const(63)
            .binop(BinaryOp::I64ShrU)
            .i64_const(1)
            .binop(BinaryOp::I64And)
            .local_set(next);

        // compute new word
        loop_
            .local_get(word)
            .i64_const(1)
            .binop(BinaryOp::I64Shl)
            .local_get(carry)
            .binop(BinaryOp::I64Or)
            .local_set(word);

        // store back
        loop_.local_get(addr).local_get(word).store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // propagate carry upward
        loop_.local_get(next).local_set(carry);

        // i++
        loop_
            .local_get(i)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(i);

        loop_
            .local_get(i)
            .i32_const(3)
            .binop(BinaryOp::I32Shl)
            .local_get(n)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_id);
    });

    function.finish(vec![a_ptr, n], &mut module.funcs)
}

fn get_bit(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I64],
    );

    let mut builder = function.name(F_GET_BIT.to_owned()).func_body();

    // Function arguments
    let ptr = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let n = module.locals.add(ValType::I32);

    let byte_idx = module.locals.add(ValType::I32);
    let bit_offset = module.locals.add(ValType::I32);

    builder
        .local_get(i)
        .i32_const(3)
        .binop(BinaryOp::I32ShrU)
        .local_set(byte_idx);

    // Bit offset
    builder
        .local_get(i)
        .i32_const(7)
        .binop(BinaryOp::I32And)
        .local_set(bit_offset);

    // Load the byte from memory: *(ptr + byteIndex)
    builder
        .local_get(ptr)
        .local_get(byte_idx)
        .binop(BinaryOp::I32Add)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Extract the bit: (byteVal >> bitOffset) & 1
    builder
        .local_get(bit_offset)
        .binop(BinaryOp::I32ShrU)
        .i32_const(1)
        .binop(BinaryOp::I32And)
        .unop(walrus::ir::UnaryOp::I64ExtendUI32);

    function.finish(vec![ptr, i, n], &mut module.funcs)
}

fn set_bit(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder = function.name(F_SET_BIT.to_owned()).func_body();

    // Function arguments
    let ptr = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);
    let n = module.locals.add(ValType::I32);

    let byte_idx = module.locals.add(ValType::I32);
    let bit_offset = module.locals.add(ValType::I32);
    let addr = module.locals.add(ValType::I32);
    let byte_val = module.locals.add(ValType::I32);
    let mask = module.locals.add(ValType::I32);

    builder
        .local_get(i)
        .i32_const(3)
        .binop(BinaryOp::I32ShrU)
        .local_set(byte_idx);

    // Bit offset
    builder
        .local_get(i)
        .i32_const(7)
        .binop(BinaryOp::I32And)
        .local_set(bit_offset);

    // Load the byte from memory: *(ptr + byteIndex)
    builder
        .local_get(ptr)
        .local_get(byte_idx)
        .binop(BinaryOp::I32Add)
        .local_tee(addr);

    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(byte_val);

    builder
        .i32_const(1)
        .local_get(bit_offset)
        .binop(BinaryOp::I32Shl)
        .local_set(mask);

    builder
        .local_get(addr)
        .local_get(byte_val)
        .local_get(mask)
        .binop(BinaryOp::I32Or)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function.finish(vec![ptr, i, n], &mut module.funcs)
}

#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{
        build_module, get_linker_with_host_debug_functions, setup_wasmtime_module,
    };
    use alloy_primitives::U256;
    use rstest::rstest;
    use walrus::FunctionBuilder;

    use super::*;

    #[rstest]
    #[case(350_u128, 127, 0)]
    #[case(5_u128, 127, 0)]
    #[case(1_u128, 0, 1)]
    /*
    #[case(1_u128, 2, 0)]
    #[case(1_u128, 8, 0)]
    #[case(u8::MAX as u128, 8, 0)]
    #[case(u8::MAX as u128, 6, 1)]
    #[case(u8::MAX as u128, 7, 1)]
    #[case(u8::MAX as u128, 3, 1)]
    #[case(u8::MAX as u128, 1, 1)]
    #[case(u16::MAX as u128, 8, 1)]
    #[case(u16::MAX as u128, 15, 1)]
    #[case(u16::MAX as u128, 16, 0)]
    #[case(u64::MAX as u128, 127, 0)]
    #[case(u64::MAX as u128, 65, 0)]
    #[case(u64::MAX as u128, 33, 1)]
    #[case(u64::MAX as u128, 100, 0)]
    #[case(u128::MAX, 127, 1)]
    #[case(u128::MAX, 65, 1)]
    #[case(u128::MAX, 33, 1)]
    #[case(u128::MAX, 100, 1)]
    */
    fn test_get_bit(#[case] a: u128, #[case] n: i32, #[case] expected: i64) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I64]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body.i32_const(0).i32_const(n).i32_const(128);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_64bits_right_f = get_bit(&mut raw_module, &compilation_ctx);
        func_body.call(shift_64bits_right_f);

        let function = function_builder.finish(vec![a_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = get_linker_with_host_debug_functions();

        let data = a.to_le_bytes();
        let (_, _, mut store, entrypoint) = setup_wasmtime_module::<i32, i64>(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let bit = entrypoint.call(&mut store, 0).unwrap();

        assert_eq!(expected, bit);

        println!("---------> {bit}");
    }

    #[rstest]
    #[case(1025, 1025 << 1)]
    #[case(0x80, 0x80 << 1)]
    #[case(1, 1 << 1)]
    #[case(42, 42 << 1)]
    #[case(u8::MAX as u128, (u8::MAX as u128) << 1)]
    #[case(u16::MAX as u128, (u16::MAX as u128) << 1)]
    #[case(u32::MAX as u128, (u32::MAX as u128) << 1)]
    #[case(u64::MAX as u128, (u64::MAX as u128) << 1)]
    fn test_shift_1bit_left_u128(#[case] a: u128, #[case] expected: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for shift_64bits_right (a_ptr, size in heap)
        func_body.i32_const(0).i32_const(2);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_64bits_right_f = shift_1bit_left(&mut raw_module, &compilation_ctx);
        func_body.call(shift_64bits_right_f);

        let function = function_builder.finish(vec![a_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = get_linker_with_host_debug_functions();

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

        println!("original");
        for byte in a.to_le_bytes() {
            print!("{byte:08b} ");
        }
        println!();

        println!("result {}", u128::from_le_bytes(result.try_into().unwrap()));
        for byte in result {
            print!("{byte:08b} ");
        }
        println!();

        println!("expected");
        for byte in expected.to_le_bytes() {
            print!("{byte:08b} ");
        }
        println!();

        println!("---------> {expected} {:?}", expected.to_le_bytes());
        assert_eq!(result, expected.to_le_bytes());
    }

    #[rstest]
    #[case(U256::from(1), U256::from(1) << 1)]
    #[case(U256::from(0x80), U256::from(0x80) << 1)]
    #[case(U256::from(42), U256::from(42) << 1)]
    #[case(U256::from(u8::MAX), U256::from(u8::MAX) << 1)]
    #[case(U256::from(u16::MAX), U256::from(u16::MAX) << 1)]
    #[case(U256::from(u32::MAX), U256::from(u32::MAX) << 1)]
    #[case(U256::from(u64::MAX), U256::from(u64::MAX) << 1)]
    #[case(U256::from(u128::MAX), U256::from(u128::MAX) << 1)]
    fn test_shift_1bit_left_u256(#[case] a: U256, #[case] expected: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id) = build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for shift_64bits_right (a_ptr, size in heap)
        func_body.i32_const(0).i32_const(4);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let shift_64bits_right_f = shift_1bit_left(&mut raw_module, &compilation_ctx);
        func_body.call(shift_64bits_right_f);

        let function = function_builder.finish(vec![a_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let linker = get_linker_with_host_debug_functions();
        let data = a.to_le_bytes::<32>();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module::<i32, ()>(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        entrypoint.call(&mut store, 0).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let result = &memory.data(&mut store)[0..TYPE_HEAP_SIZE as usize];

        println!("original");
        for byte in a.to_le_bytes::<32>() {
            print!("{byte:08b}");
        }
        println!();

        println!("result");
        for byte in result {
            print!("{byte:08b}");
        }
        println!();

        println!("expected");
        for byte in expected.to_le_bytes::<32>() {
            print!("{byte:08b}");
        }
        println!();

        assert_eq!(result, expected.to_le_bytes::<32>());
    }

    #[rstest]
    #[case(350, 13, 26)]
    #[case(12, 4, 3)]
    #[case(5, 2, 2)]
    #[case(123456, 1, 123456)]
    #[case(987654321, 123456789, 8)]
    #[case(0, 2, 0)]
    // 2^96 / 2^32 = 2^64
    #[case(79228162514264337593543950336, 4294967296, 18446744073709551616)]
    // #[should_panic(expected = "wasm trap: integer divide by zero")]
    // #[case(10, 0, 0)]
    #[case(u128::MAX, u64::MAX as u128 + 1, u64::MAX as u128)]
    #[case(u128::MAX, 79228162514264337593543950336, 4294967295)]
    fn test_div_u128(#[case] n1: u128, #[case] n2: u128, #[case] quotient: u128) {
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

        // arguments for heap_integers_add (n1_ptr, n2_ptr, size in heap and return quotient)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(128)
            .i32_const(1);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_div_mod(&mut raw_module, &compilation_ctx).unwrap();
        // Shift left
        func_body.call(heap_integers_add_f);

        let linker = get_linker_with_host_debug_functions();
        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);
        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let quotient_ptr: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(
                &mut store,
                quotient_ptr as usize,
                &mut quotient_result_memory_data,
            )
            .unwrap();

        assert_eq!(quotient_result_memory_data, quotient.to_le_bytes());
    }

    #[rstest]
    #[case(350, 13, 12)]
    #[case(5, 2, 1)]
    #[case(123456, 1, 0)]
    #[case(987654321, 123456789, 9)]
    #[case(0, 2, 0)]
    // 2^96 % 2^32 = 0
    #[case(79228162514264337593543950336, 4294967296, 0)]
    // #[should_panic(expected = "wasm trap: integer divide by zero")]
    //#[case(10, 0, 0)]
    // (2^128 - 1) % 2^64 = 2^64 - 1
    #[case(u128::MAX, u64::MAX as u128 + 1, u64::MAX as u128)]
    #[case(
        u128::MAX,
        79228162514264337593543950336,
        79_228_162_514_264_337_593_543_950_335
    )]
    fn test_mod_u128(#[case] n1: u128, #[case] n2: u128, #[case] remainder: u128) {
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

        // arguments for heap_integers_add (n1_ptr, n2_ptr, size in heap and return remainder)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            // .i32_const(TYPE_HEAP_SIZE)
            .i32_const(128)
            .i32_const(0);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_div_mod(&mut raw_module, &compilation_ctx).unwrap();
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);
        let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();

        let linker = get_linker_with_host_debug_functions();
        let (_, instance, mut store, entrypoint) = setup_wasmtime_module(
            &mut raw_module,
            data.to_vec(),
            "test_function",
            Some(linker),
        );

        let remainder_ptr: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let mut remainder_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(
                &mut store,
                remainder_ptr as usize,
                &mut remainder_result_memory_data,
            )
            .unwrap();

        assert_eq!(remainder_result_memory_data, remainder.to_le_bytes());
    }

    #[rstest]
    #[case(U256::from(350), U256::from(13), U256::from(26))]
    #[case(U256::from(5), U256::from(2), U256::from(2))]
    #[case(U256::from(123456), U256::from(1), U256::from(123456))]
    #[case(U256::from(987654321), U256::from(123456789), U256::from(8))]
    #[case(U256::from(0), U256::from(2), U256::from(0))]
    // 2^96 / 2^32 = 2^64
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(4294967296_u128),
        U256::from(18446744073709551616_u128)
    )]
    // 2^192 / 2^64 = 2^128
    #[case(
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551616_u128),
        U256::from(u128::MAX) + U256::from(1),
    )]
    // Timeouts, the algorithm is slow yet
    // (2^128 - 1) / 2^64 = 2^64 - 1
    // #[case(u128::MAX, u64::MAX as u128 + 1, u64::MAX as u128, u64::MAX as u128)]
    // #[case(u128::MAX, 79228162514264337593543950336, u64::MAX as u128, u64::MAX as u128)]
    fn test_div_u256(#[case] n1: U256, #[case] n2: U256, #[case] quotient: U256) {
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

        // arguments for heap_integers_add (n1_ptr, n2_ptr, size in heap and return quotient)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(1);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_div_mod(&mut raw_module, &compilation_ctx).unwrap();
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1.to_le_bytes::<32>(), n2.to_le_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let quotient_ptr: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(
                &mut store,
                quotient_ptr as usize,
                &mut quotient_result_memory_data,
            )
            .unwrap();

        assert_eq!(quotient_result_memory_data, quotient.to_le_bytes::<32>());
    }

    #[rstest]
    #[case(U256::from(350), U256::from(13), U256::from(12))]
    #[case(U256::from(5), U256::from(2), U256::from(1))]
    #[case(U256::from(123456), U256::from(1), U256::from(0))]
    #[case(U256::from(987654321), U256::from(123456789), U256::from(9))]
    #[case(U256::from(0), U256::from(2), U256::from(0))]
    // 2^96 % 2^32 = 0
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(4294967296_u128),
        U256::from(0)
    )]
    // 2^192 % 2^64 = 0
    #[case(
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551616_u128),
        U256::from(0)
    )]
    // Timeouts, the algorithm is slow yet
    // (2^128 - 1) / 2^64 = [q = 2^64 - 1, r = 2^64 - 1]
    // #[case(u128::MAX, u64::MAX as u128 + 1, u64::MAX as u128, u64::MAX as u128)]
    // #[case(u128::MAX, 79228162514264337593543950336, u64::MAX as u128, u64::MAX as u128)]
    fn test_mod_u256(#[case] n1: U256, #[case] n2: U256, #[case] remainder: U256) {
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

        // arguments for heap_integers_add (n1_ptr, n2_ptr, size in heap and return remainder)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(0);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func);
        let heap_integers_add_f = heap_integers_div_mod(&mut raw_module, &compilation_ctx).unwrap();
        // Shift left
        func_body.call(heap_integers_add_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1.to_le_bytes::<32>(), n2.to_le_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let remainder_ptr: i32 = entrypoint.call(&mut store, (0, TYPE_HEAP_SIZE)).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let mut remainder_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
        memory
            .read(
                &mut store,
                remainder_ptr as usize,
                &mut remainder_result_memory_data,
            )
            .unwrap();

        assert_eq!(remainder_result_memory_data, remainder.to_le_bytes::<32>());
    }
}

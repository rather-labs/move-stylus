use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    data::RuntimeErrorData,
    runtime::{RuntimeFunction, error::RuntimeFunctionError},
};

// Auxiliary function names
const F_SHIFT_64BITS_LEFT: &str = "shift_64bits_left";
const F_GET_BIT: &str = "get_bit";
const F_SET_BIT: &str = "set_bit";

/// Implements the long division algorithm for unsigned 128 and 256 bit integers.
///
/// if D = 0 then error(DivisionByZeroException) end
/// Q := 0                  -- Initialize quotient and remainder to zero
/// R := 0
/// for i := n − 1 .. 0 do  -- Where n is number of bits in N
///   R := R << 1           -- Left-shift R by 1 bit
///   R(0) := N(i)          -- Set the least-significant bit of R equal to bit i of the numerator
///   if R ≥ D then
///     R := R − D
///     Q(i) := 1
///   end
/// end
///
/// **Note:** In the implementation, indices and operations are changed because we work in
/// little-endian format. This description and the example assume big-endian for clarity.
///
/// # WASM Function Arguments
/// * `dividend_ptr` (i32) - Pointer to the dividend
/// * `divisor_ptr` (i32) - Pointer to the divisor
/// * `num_bits` (i32) - Number of bits the values occupy in memory
/// * `return_quotient` (i32) - Whether return remainder or quotient. 1 for quotient, 0 for remainder.
///
/// # WASM Function Returns
/// * Pointer to the result
pub fn heap_integers_div_mod(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let check_if_a_less_than_b_f =
        RuntimeFunction::LessThan.get(module, Some(compilation_ctx), None)?;
    let sub_f =
        RuntimeFunction::HeapIntSub.get(module, Some(compilation_ctx), Some(runtime_error_data))?;
    let is_zero = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
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

    let mut builder = function
        .name(RuntimeFunction::HeapIntDivMod.name().to_owned())
        .func_body();

    builder
        .local_get(n)
        .i32_const(8)
        .binop(BinaryOp::I32DivU)
        .local_set(n_bytes);

    // If the division is zero, we provoke a division by zero runtime error
    builder
        .local_get(divisor_ptr)
        .local_get(n_bytes)
        .call(is_zero)
        .if_else(
            None,
            |then_| {
                then_
                    .i32_const(0)
                    .i32_const(0)
                    .binop(BinaryOp::I32DivU)
                    .drop();
            },
            |_| {},
        );

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

/// Shifts left by 1 bit a number stored in memory in little-endian format.
///
/// Given that the number is in little-endian format, the least significant bytes are stored first,
/// but the least significant bit of each byte is the rightmost one, therefore
///
/// LSB 10000000 00000000 MSB << 1 is
/// LSB 00000000 00000001 MSB
///
/// for each byte that composes the number
///
/// # WASM Function Arguments
/// * `ptr` (i32) - number pointer
/// * `n` (i32) - number of bytes the number occupies in memory
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

/// Gets the bit at index i of a number stored in memory in little-endian format.
///
/// This operation is logical, meaning that 0 is the least significant bit while n-1 is the most
/// significant bit. For example, to get the least significant bit of a number, i = 0, we get the
/// firt byte and we extract the 7th bit (from 0 to 7). This conversion is done inside the
/// function.
///
/// # WASM Function Arguments
/// * `ptr` (i32) - number pointer
/// * `i` (i32) - bit index
/// * `n` (i32) - number of bits the number occupies in memory
///
/// # WASM Function Returns
/// * bit value (0 or 1)
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

/// Sets the bit at index i of a number stored in memory in little-endian format.
///
/// This operation is logical, meaning that 0 is the least significant bit while n-1 is the most
/// significant bit. For example, to set the least significant bit of a number, i = 0, we get the
/// firt byte and we set the 7th bit (from 0 to 7). This conversion is done inside the
/// function.
///
/// # WASM Function Arguments
/// * `ptr` (i32) - number pointer
/// * `i` (i32) - bit index
/// * `n` (i32) - number of bits the number occupies in memory
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
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;

    use crate::test_compilation_context;
    use crate::test_runtime_error_data;
    use crate::test_tools::{
        INITIAL_MEMORY_OFFSET, build_module, get_linker_with_host_debug_functions,
        setup_wasmtime_module,
    };
    use alloy_primitives::U256;
    use rstest::rstest;
    use walrus::{FunctionBuilder, ValType};

    use super::*;

    #[rstest]
    #[case(350_u128, 127, 0)]
    #[case(5_u128, 127, 0)]
    #[case(1_u128, 0, 1)]
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
    fn test_get_bit(#[case] a: u128, #[case] n: i32, #[case] expected: i64) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I64]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .i32_const(n)
            .i32_const(128);

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
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for shift_64bits_right (a_ptr, size in heap)
        func_body.i32_const(INITIAL_MEMORY_OFFSET).i32_const(16);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
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
        let result = &memory.data(&mut store)
            [INITIAL_MEMORY_OFFSET as usize..(INITIAL_MEMORY_OFFSET + TYPE_HEAP_SIZE) as usize];

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
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE));

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[]);

        let a_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for shift_64bits_right (a_ptr, size in heap)
        func_body.i32_const(INITIAL_MEMORY_OFFSET).i32_const(32);

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
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
        let result = &memory.data(&mut store)
            [INITIAL_MEMORY_OFFSET as usize..(INITIAL_MEMORY_OFFSET + TYPE_HEAP_SIZE) as usize];

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
    #[case(u128::MAX, u64::MAX as u128 + 1, u64::MAX as u128)]
    #[case(u128::MAX, 79228162514264337593543950336, 4294967295)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_div_u128(#[case] n1: u128, #[case] n2: u128, #[case] quotient: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_div (n1_ptr, n2_ptr, size in heap and return quotient)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(128)
            .i32_const(1);

        let heap_integers_div_f =
            heap_integers_div_mod(&mut raw_module, &compilation_ctx, &mut runtime_error_data)
                .unwrap();
        func_body.call(heap_integers_div_f);

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

    #[test]
    pub fn test_div_u128_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type()
            .for_each(|&(n1, n2): &(u128, u128)| {
                let mut store = store.borrow_mut();
                let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
                memory
                    .write(&mut *store, INITIAL_MEMORY_OFFSET as usize, &data)
                    .unwrap();

                let expected_quotient = n1 / n2;

                let quotient_ptr: Result<i32, _> =
                    entrypoint.call(&mut *store, (0, TYPE_HEAP_SIZE));

                match quotient_ptr {
                    Ok(quotient_ptr) => {
                        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                        memory
                            .read(
                                &mut *store,
                                quotient_ptr as usize,
                                &mut quotient_result_memory_data,
                            )
                            .unwrap();

                        assert_eq!(quotient_result_memory_data, expected_quotient.to_le_bytes());
                    }
                    Err(_) => {
                        // Division by zero
                        assert_eq!(n2, 0);
                    }
                }

                reset_memory.call(&mut *store, ()).unwrap();
            });
    }

    #[rstest]
    #[case(350, 13, 12)]
    #[case(5, 2, 1)]
    #[case(123456, 1, 0)]
    #[case(987654321, 123456789, 9)]
    #[case(0, 2, 0)]
    // 2^96 % 2^32 = 0
    #[case(79228162514264337593543950336, 4294967296, 0)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    // (2^128 - 1) % 2^64 = 2^64 - 1
    #[case(u128::MAX, u64::MAX as u128 + 1, u64::MAX as u128)]
    #[case(
        u128::MAX,
        79228162514264337593543950336,
        79_228_162_514_264_337_593_543_950_335
    )]
    fn test_mod_u128(#[case] n1: u128, #[case] n2: u128, #[case] remainder: u128) {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // arguments for heap_integers_div (n1_ptr, n2_ptr, size in heap and return remainder)
        func_body
            .i32_const(0)
            .i32_const(TYPE_HEAP_SIZE)
            .i32_const(128)
            .i32_const(0);

        let heap_integers_div_f =
            heap_integers_div_mod(&mut raw_module, &compilation_ctx, &mut runtime_error_data)
                .unwrap();
        // Shift left
        func_body.call(heap_integers_div_f);

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

    #[test]
    pub fn test_mod_u128_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 16;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type()
            .for_each(|&(n1, n2): &(u128, u128)| {
                let mut store = store.borrow_mut();
                let data = [n1.to_le_bytes(), n2.to_le_bytes()].concat();
                memory
                    .write(&mut *store, INITIAL_MEMORY_OFFSET as usize, &data)
                    .unwrap();

                let expected_quotient = n1 % n2;

                let reminder_ptr: Result<i32, _> =
                    entrypoint.call(&mut *store, (0, TYPE_HEAP_SIZE));

                match reminder_ptr {
                    Ok(quotient_ptr) => {
                        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                        memory
                            .read(
                                &mut *store,
                                quotient_ptr as usize,
                                &mut quotient_result_memory_data,
                            )
                            .unwrap();

                        assert_eq!(quotient_result_memory_data, expected_quotient.to_le_bytes());
                    }
                    Err(_) => {
                        // Division by zero
                        assert_eq!(n2, 0);
                    }
                }

                reset_memory.call(&mut *store, ()).unwrap();
            });
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
    #[case(U256::from(u128::MAX), U256::from(u64::MAX as u128 + 1), U256::from(u64::MAX as u128))]
    #[case(
        U256::from(u128::MAX),
        U256::from(79228162514264337593543950336_u128),
        U256::from(4294967295_u128)
    )]
    #[case(
        U256::MAX,
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551615_u128),
    )]
    fn test_div_u256(#[case] n1: U256, #[case] n2: U256, #[case] quotient: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let (mut store, instance, entrypoint) = setup_heap_div_test(
            n1.to_le_bytes::<32>().to_vec(),
            n2.to_le_bytes::<32>().to_vec(),
            TYPE_HEAP_SIZE,
            true,
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

        assert_eq!(quotient_result_memory_data, quotient.to_le_bytes::<32>());
    }

    #[test]
    pub fn test_div_u256_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type()
            .for_each(|&(n1, n2): &([u8; 32], [u8; 32])| {
                let mut store = store.borrow_mut();
                let data = [n1, n2].concat();

                memory
                    .write(&mut *store, INITIAL_MEMORY_OFFSET as usize, &data)
                    .unwrap();

                let n1 = U256::from_le_bytes(n1);
                let n2 = U256::from_le_bytes(n2);

                let expected_quotient = n1 / n2;

                let quotient_ptr: Result<i32, _> =
                    entrypoint.call(&mut *store, (0, TYPE_HEAP_SIZE));

                match quotient_ptr {
                    Ok(quotient_ptr) => {
                        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                        memory
                            .read(
                                &mut *store,
                                quotient_ptr as usize,
                                &mut quotient_result_memory_data,
                            )
                            .unwrap();

                        assert_eq!(
                            quotient_result_memory_data,
                            expected_quotient.to_le_bytes::<32>()
                        );
                    }
                    Err(_) => {
                        // Division by zero
                        assert_eq!(n2, 0);
                    }
                }

                reset_memory.call(&mut *store, ()).unwrap();
            });
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
    #[case(U256::from(u128::MAX), U256::from(u64::MAX as u128 + 1), U256::from(u64::MAX as u128))]
    #[case(
        U256::from(u128::MAX),
        U256::from(79228162514264337593543950336_u128),
        U256::from(79228162514264337593543950335_u128)
    )]
    #[case(U256::MAX, U256::from(u128::MAX) + U256::from(1), U256::from(u128::MAX))]
    #[case(
        U256::MAX,
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512895", 10
        ).unwrap()
    )]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(U256::from(10), U256::from(0), U256::from(0))]
    fn test_mod_u256(#[case] n1: U256, #[case] n2: U256, #[case] remainder: U256) {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
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

        assert_eq!(remainder_result_memory_data, remainder.to_le_bytes::<32>());
    }

    #[test]
    pub fn test_mod_u256_fuzz() {
        const TYPE_HEAP_SIZE: i32 = 32;
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(TYPE_HEAP_SIZE * 2));

        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();
        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type()
            .for_each(|&(n1, n2): &([u8; 32], [u8; 32])| {
                let mut store = store.borrow_mut();
                let data = [n1, n2].concat();

                memory
                    .write(&mut *store, INITIAL_MEMORY_OFFSET as usize, &data)
                    .unwrap();

                let n1 = U256::from_le_bytes(n1);
                let n2 = U256::from_le_bytes(n2);

                let expected_quotient = n1 % n2;

                let remainder_ptr: Result<i32, _> =
                    entrypoint.call(&mut *store, (0, TYPE_HEAP_SIZE));

                match remainder_ptr {
                    Ok(quotient_ptr) => {
                        let mut quotient_result_memory_data = vec![0; TYPE_HEAP_SIZE as usize];
                        memory
                            .read(
                                &mut *store,
                                quotient_ptr as usize,
                                &mut quotient_result_memory_data,
                            )
                            .unwrap();

                        assert_eq!(
                            quotient_result_memory_data,
                            expected_quotient.to_le_bytes::<32>()
                        );
                    }
                    Err(_) => {
                        // Division by zero
                        assert_eq!(n2, 0);
                    }
                }

                reset_memory.call(&mut *store, ()).unwrap();
            });
    }

    fn setup_heap_div_test(
        n1_bytes: Vec<u8>,
        n2_bytes: Vec<u8>,
        heap_size: i32,
        return_quotient: bool,
    ) -> (
        wasmtime::Store<()>,
        wasmtime::Instance,
        wasmtime::TypedFunc<(i32, i32), i32>,
    ) {
        let (mut raw_module, allocator_func, memory_id, ctx_globals) =
            build_module(Some(heap_size * 2));
        let compilation_ctx = test_compilation_context!(memory_id, allocator_func, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder = FunctionBuilder::new(
            &mut raw_module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        let n1_ptr = raw_module.locals.add(ValType::I32);
        let n2_ptr = raw_module.locals.add(ValType::I32);

        let div_f =
            heap_integers_div_mod(&mut raw_module, &compilation_ctx, &mut runtime_error_data)
                .unwrap();

        let mut func_body = function_builder.func_body();

        func_body
            .i32_const(INITIAL_MEMORY_OFFSET)
            .i32_const(INITIAL_MEMORY_OFFSET + heap_size);

        if heap_size == 16 {
            func_body.i32_const(128);
        } else {
            func_body.i32_const(256);
        }

        if return_quotient {
            func_body.i32_const(1);
        } else {
            func_body.i32_const(0);
        }

        func_body.call(div_f);

        let function = function_builder.finish(vec![n1_ptr, n2_ptr], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let data = [n1_bytes, n2_bytes].concat();
        let (_, instance, store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data, "test_function", None);

        (store, instance, entrypoint)
    }
}

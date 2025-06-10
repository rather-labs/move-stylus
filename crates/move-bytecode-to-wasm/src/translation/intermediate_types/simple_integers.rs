use walrus::{
    InstrSeqBuilder,
    ir::{BinaryOp, UnaryOp},
};

use crate::{
    CompilationContext,
    runtime::RuntimeFunction,
    wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions},
};

use super::{
    IntermediateType,
    heap_integers::{IU128, IU256},
};

#[derive(Clone, Copy)]
pub struct IU8;

impl IU8 {
    const MAX_VALUE: i32 = u8::MAX as i32;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(1).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    /// Adds the instructions to add two u8 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than 255
    /// then the execution is aborted This check is posible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder
            .binop(BinaryOp::I32Add)
            .i32_const(Self::MAX_VALUE)
            .call(check_overflow_f);
    }

    /// Adds the instructions to substract two u8 values.
    ///
    /// If the substraction is less than 0, then it traps
    pub fn sub(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let sub_u32_f = RuntimeFunction::SubU32.get(module, None);
        builder.call(sub_u32_f);
    }

    /// Adds the instructions to divide two u8 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn div(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I32DivU);
    }

    /// Adds the instructions to calculate the remainder between two u8 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn remainder(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I32RemU);
    }

    /// Adds the instructions to multiply two u8 values.
    ///
    /// Along with the multiplication code to check overflow is added. If the result is greater
    /// than 255 then the execution is aborted. This check is posible because interally we are
    /// using 32bits integers.
    pub fn mul(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder
            .binop(BinaryOp::I32Mul)
            .i32_const(Self::MAX_VALUE)
            .call(check_overflow_f);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 => {
                return;
            }
            // Just check for overflow and leave the value in the stack again
            IntermediateType::IU16 | IntermediateType::IU32 => {}
            IntermediateType::IU64 => {
                builder.unop(UnaryOp::I32WrapI64);
            }
            IntermediateType::IU128 => {
                let downcast_u128_u256_to_u32_f =
                    RuntimeFunction::DowncastU128U256ToU32.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u32_f =
                    RuntimeFunction::DowncastU128U256ToU32.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }

        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(Self::MAX_VALUE).call(check_overflow_f);
    }

    pub fn bit_shift_left(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 8
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(7).call(check_overflow_f);

        builder.binop(BinaryOp::I32Shl);
        // Mask the bytes outside the u8 range
        builder.i32_const(0xFF).binop(BinaryOp::I32And);
    }

    pub fn bit_shift_right(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 8
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(7).call(check_overflow_f);

        builder.binop(BinaryOp::I32ShrU);
    }
}

#[derive(Clone, Copy)]
pub struct IU16;

impl IU16 {
    const MAX_VALUE: i32 = u16::MAX as i32;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(2).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    /// Adds the instructions to add two u16 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 65535 then the execution is aborted. This check is posible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder
            .binop(BinaryOp::I32Add)
            .i32_const(Self::MAX_VALUE)
            .call(check_overflow_f);
    }

    /// Adds the instructions to substract two u16 values.
    ///
    /// If the substraction is less than 0, then it traps
    pub fn sub(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let sub_u32_f = RuntimeFunction::SubU32.get(module, None);
        builder.call(sub_u32_f);
    }

    /// Adds the instructions to divide two u16 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn div(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I32DivU);
    }

    /// Adds the instructions to calculate the remainder between two u16 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn remainder(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I32RemU);
    }

    /// Adds the instructions to multiply two u16 values.
    ///
    /// Along with the multiplication code to check overflow is added. If the result is greater
    /// than u16::MAX then the execution is aborted. This check is posible because interally we are
    /// using 32bits integers.
    pub fn mul(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder
            .binop(BinaryOp::I32Mul)
            .i32_const(Self::MAX_VALUE)
            .call(check_overflow_f);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 => {
                return;
            }
            // Just check for overflow and leave the value in the stack again
            IntermediateType::IU32 => {}
            IntermediateType::IU64 => {
                builder.unop(UnaryOp::I32WrapI64);
            }
            IntermediateType::IU128 => {
                let downcast_u128_u256_to_u32_f =
                    RuntimeFunction::DowncastU128U256ToU32.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u32_f =
                    RuntimeFunction::DowncastU128U256ToU32.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }

        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(Self::MAX_VALUE).call(check_overflow_f);
    }

    pub fn bit_shift_left(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 16
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(15).call(check_overflow_f);

        builder.binop(BinaryOp::I32Shl);
        // Mask the bytes outside the u16 range
        builder.i32_const(0xFFFF).binop(BinaryOp::I32And);
    }

    pub fn bit_shift_right(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 16
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(15).call(check_overflow_f);

        builder.binop(BinaryOp::I32ShrU);
    }
}

#[derive(Clone, Copy)]
pub struct IU32;

impl IU32 {
    pub const MAX_VALUE: i64 = u32::MAX as i64;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(4).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let add_function_id = RuntimeFunction::AddU32.get(module, None);
        builder.call(add_function_id);
    }

    /// Adds the instructions to substract two u32 values.
    ///
    /// If the substraction is less than 0, then it traps
    pub fn sub(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let sub_u32_f = RuntimeFunction::SubU32.get(module, None);
        builder.call(sub_u32_f);
    }

    /// Adds the instructions to divide two u32 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn div(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I32DivU);
    }

    /// Adds the instructions to calculate the remainder between two u32 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn remainder(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I32RemU);
    }

    /// Adds the instructions to multiply two u32 values.
    ///
    /// Along with the multiplication code to check overflow is added. If the result is greater
    /// than u32::MAX then the execution is aborted. This check is posible because interally we are
    /// using 32bits integers.
    pub fn mul(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let mul_f = RuntimeFunction::MulU32.get(module, None);
        builder.call(mul_f);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {}
            IntermediateType::IU64 => {
                let downcast_u64_to_u32_f = RuntimeFunction::DowncastU64ToU32.get(module, None);
                builder.call(downcast_u64_to_u32_f);
            }
            IntermediateType::IU128 => {
                let downcast_u128_u256_to_u32_f =
                    RuntimeFunction::DowncastU128U256ToU32.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u32_f =
                    RuntimeFunction::DowncastU128U256ToU32.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }

    pub fn bit_shift_left(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 32
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(31).call(check_overflow_f);

        builder.binop(BinaryOp::I32Shl);
    }

    pub fn bit_shift_right(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 32
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(31).call(check_overflow_f);

        builder.binop(BinaryOp::I32ShrU);
    }
}

#[derive(Clone, Copy)]
pub struct IU64;

impl IU64 {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(8).collect::<Vec<u8>>();
        load_i64_from_bytes_instructions(builder, &bytes);
    }

    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let add_function_id = RuntimeFunction::AddU64.get(module, None);
        builder.call(add_function_id);
    }

    /// Adds the instructions to substract two u8 values.
    ///
    /// If the substraction is less than 0, then it traps
    pub fn sub(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let sub_u64_f = RuntimeFunction::SubU64.get(module, None);
        builder.call(sub_u64_f);
    }

    /// Adds the instructions to divide two u64 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn div(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I64DivU);
    }

    /// Adds the instructions to calculate the remainder between two u64 values.
    ///
    /// If the dividend is 0, then it traps
    pub fn remainder(builder: &mut walrus::InstrSeqBuilder) {
        builder.binop(BinaryOp::I64RemU);
    }

    /// Adds the instructions to multiply two u64 values.
    ///
    /// Along with the multiplication code to check overflow is added. If the result is greater
    /// than u64::MAX then the execution is aborted. This check is posible because interally we are
    /// using 64bits integers.
    pub fn mul(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let mul_f = RuntimeFunction::MulU64.get(module, None);
        builder.call(mul_f);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                builder.unop(UnaryOp::I64ExtendUI32);
            }
            IntermediateType::IU64 => {}
            IntermediateType::IU128 => {
                let downcast_u128_u256_to_u64_f =
                    RuntimeFunction::DowncastU128U256ToU64.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u64_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u64_f =
                    RuntimeFunction::DowncastU128U256ToU64.get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u64_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }

    pub fn bit_shift_left(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 64
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(63).call(check_overflow_f);

        builder.unop(UnaryOp::I64ExtendUI32);
        builder.binop(BinaryOp::I64Shl);
    }

    pub fn bit_shift_right(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        // This operation aborts if the shift amount is greater or equal than 64
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(63).call(check_overflow_f);

        builder.unop(UnaryOp::I64ExtendUI32);
        builder.binop(BinaryOp::I64ShrU);
    }
}

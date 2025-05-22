use walrus::{
    ir::{BinaryOp, UnaryOp},
    InstrSeqBuilder,
};

use crate::{
    runtime::RuntimeFunction,
    wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions},
    CompilationContext,
};

use super::{
    heap_integers::{IU128, IU256},
    IntermediateType,
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
    /// then the execution is aborted This check is poosible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder
            .binop(BinaryOp::I32Add)
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
                let downcast_u128_u256_to_u32_f = RuntimeFunction::DowncastU128U256ToU32
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u32_f = RuntimeFunction::DowncastU128U256ToU32
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }

        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(Self::MAX_VALUE).call(check_overflow_f);
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
    /// 65535 then the execution is aborted. This check is poosible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder
            .binop(BinaryOp::I32Add)
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
                let downcast_u128_u256_to_u32_f = RuntimeFunction::DowncastU128U256ToU32
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u32_f = RuntimeFunction::DowncastU128U256ToU32
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }

        let check_overflow_f = RuntimeFunction::CheckOverflowU8U16.get(module, None);
        builder.i32_const(Self::MAX_VALUE).call(check_overflow_f);
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

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {}
            IntermediateType::IU64 => {
                let downcast_u64_to_u32_f =
                    RuntimeFunction::DowncastU64ToU32.get(module, None);
                builder.call(downcast_u64_to_u32_f);
            }
            IntermediateType::IU128 => {
                let downcast_u128_u256_to_u32_f = RuntimeFunction::DowncastU128U256ToU32
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u32_f = RuntimeFunction::DowncastU128U256ToU32
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u32_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
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
                let downcast_u128_u256_to_u64_f = RuntimeFunction::DowncastU128U256ToU64
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u64_f);
            }
            IntermediateType::IU256 => {
                let downcast_u128_u256_to_u64_f = RuntimeFunction::DowncastU128U256ToU64
                    .get(module, Some(compilation_ctx));
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(downcast_u128_u256_to_u64_f);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }
}

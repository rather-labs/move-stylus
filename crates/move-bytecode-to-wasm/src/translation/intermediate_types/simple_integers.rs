use walrus::{ir::BinaryOp, InstrSeqBuilder};

use crate::wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions};

macro_rules! impl_add {
    ($valtype: expr, $add_op: expr, $gt_op: expr, $max_value: expr, $const_op: ident) => {
        pub fn add(
            builder: &mut walrus::InstrSeqBuilder,
            module_locals: &mut walrus::ModuleLocals,
        ) {
            let tmp = module_locals.add($valtype);
            builder.binop($add_op);
            builder.local_tee(tmp);
            builder.$const_op($max_value);
            builder.binop($gt_op);
            builder.if_else(
                Some($valtype),
                |then| {
                    then.unreachable();
                },
                |else_| {
                    else_.local_get(tmp);
                },
            );
        }
    };
}

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

    impl_add!(
        walrus::ValType::I32,
        walrus::ir::BinaryOp::I32Add,
        BinaryOp::I32GtU,
        Self::MAX_VALUE,
        i32_const
    );
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

    impl_add!(
        walrus::ValType::I32,
        walrus::ir::BinaryOp::I32Add,
        BinaryOp::I32GtU,
        Self::MAX_VALUE,
        i32_const
    );
}

#[derive(Clone, Copy)]
pub struct IU32;

impl IU32 {
    const MAX_VALUE: i32 = u32::MAX as i32;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(4).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    impl_add!(
        walrus::ValType::I32,
        walrus::ir::BinaryOp::I32Add,
        BinaryOp::I32GtU,
        Self::MAX_VALUE,
        i32_const
    );
}

#[derive(Clone, Copy)]
pub struct IU64;

impl IU64 {
    const MAX_VALUE: i64 = u64::MAX as i64;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(8).collect::<Vec<u8>>();
        load_i64_from_bytes_instructions(builder, &bytes);
    }

    impl_add!(
        walrus::ValType::I64,
        walrus::ir::BinaryOp::I64Add,
        BinaryOp::I64GtU,
        Self::MAX_VALUE,
        i64_const
    );
}

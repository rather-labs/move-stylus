use walrus::{ir::BinaryOp, InstrSeqBuilder, ModuleLocals};

use crate::wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions};

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

    pub fn add(builder: &mut InstrSeqBuilder, module_locals: &mut ModuleLocals) {
        let tmp = module_locals.add(walrus::ValType::I32);
        builder.binop(BinaryOp::I32Add);
        builder.local_tee(tmp);
        builder.i32_const(Self::MAX_VALUE);
        builder.binop(BinaryOp::I32GtU);
        builder.if_else(
            Some(walrus::ValType::I32),
            |then| {
                then.unreachable();
            },
            |else_| {
                else_.local_get(tmp);
            },
        );
    }
}

#[derive(Clone, Copy)]
pub struct IU16;

impl IU16 {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(2).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU32;

impl IU32 {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(4).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
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
}

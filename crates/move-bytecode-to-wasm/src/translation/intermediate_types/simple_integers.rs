use walrus::{ir::BinaryOp, InstrSeqBuilder, ValType};

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

    /// Adds the instructions to add two u8 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than 255
    /// then the execution is aborted This check is poosible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module_locals: &mut walrus::ModuleLocals) {
        let tmp = module_locals.add(walrus::ValType::I32);
        builder.binop(BinaryOp::I32Add);
        builder.local_tee(tmp);
        builder.i32_const(Self::MAX_VALUE);
        builder.binop(BinaryOp::I32GtU);
        builder.if_else(
            Some(ValType::I32),
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
    const MAX_VALUE: i32 = u16::MAX as i32;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(2).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    /// Adds the instructions to add two u8 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 65535 then the execution is aborted. This check is poosible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module_locals: &mut walrus::ModuleLocals) {
        let tmp = module_locals.add(walrus::ValType::I32);
        builder.binop(BinaryOp::I32Add);
        builder.local_tee(tmp);
        builder.i32_const(Self::MAX_VALUE);
        builder.binop(BinaryOp::I32GtU);
        builder.if_else(
            Some(ValType::I32),
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
pub struct IU32;

impl IU32 {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(4).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    /// Adds the instructions to add two u8 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 4_294_967_295 then the execution is aborted. To check the overflow we check that the result
    /// is strictly greater than the two operands. Because we are using i32 integer, if the
    /// addition overflow, WASM wraps around the result.
    ///
    /// NOTE: We use two temporal local variables to do the checks (n1, n2). If a program contains
    /// a lot of additions we will add two local variables per addition. We can optimize this by
    /// tracking and reuse the same ones used in the first addition found.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module_locals: &mut walrus::ModuleLocals) {
        let res = module_locals.add(ValType::I32);
        let n1 = module_locals.add(ValType::I32);
        let n2 = module_locals.add(ValType::I32);

        // Set the two opends to local variables and reinsert them to the stack to operate them
        builder.local_set(n1);
        builder.local_set(n2);

        builder.local_get(n1);
        builder.local_get(n2);

        builder.binop(BinaryOp::I32Add);

        // We check that the result is greater than the two operands. If this check fails means
        // WASM an overflow occured.
        // if (res > n1) && (res > n2)
        // then return res
        // else trap
        builder.local_tee(res);
        builder.local_get(n1);
        builder.binop(BinaryOp::I32GtU);
        builder.local_get(res);
        builder.local_get(n2);
        builder.binop(BinaryOp::I32GtU);
        builder.binop(BinaryOp::I32And);
        builder.if_else(
            Some(ValType::I32),
            |then| {
                then.local_get(res);
            },
            |else_| {
                else_.unreachable();
            },
        );
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

    /// Adds the instructions to add two u8 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 18_446_744_073_709_551_615 then the execution is aborted. To check the overflow we check
    /// that the result is strictly greater than the two operands. Because we are using i32
    /// integer, if the addition overflow, WASM wraps around the result.
    ///
    /// NOTE: We use two temporal local variables to do the checks (n1, n2). If a program contains
    /// a lot of additions we will add two local variables per addition. We can optimize this by
    /// tracking and reuse the same ones used in the first addition found.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module_locals: &mut walrus::ModuleLocals) {
        let res = module_locals.add(ValType::I64);
        let n1 = module_locals.add(ValType::I64);
        let n2 = module_locals.add(ValType::I64);

        // Set the two opends to local variables and reinsert them to the stack to operate them
        builder.local_set(n1);
        builder.local_set(n2);

        builder.local_get(n1);
        builder.local_get(n2);

        // We check that the result is greater than the two operands. If this check fails means
        // WASM an overflow occured.
        // if (res > n1) && (res > n2)
        // then return res
        // else trap
        builder.binop(BinaryOp::I64Add);
        builder.local_tee(res);
        builder.local_get(n1);
        builder.binop(BinaryOp::I64GtU);
        builder.local_get(res);
        builder.local_get(n2);
        builder.binop(BinaryOp::I64GtU);
        builder.binop(BinaryOp::I32And);
        builder.if_else(
            Some(ValType::I64),
            |then| {
                then.local_get(res);
            },
            |else_| {
                else_.unreachable();
            },
        );
    }
}

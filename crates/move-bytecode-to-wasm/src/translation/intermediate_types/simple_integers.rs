use walrus::{
    ir::{BinaryOp, LoadKind, MemArg, UnaryOp},
    InstrSeqBuilder, MemoryId, ValType,
};

use crate::wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions};

use super::IntermediateType;

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

    fn add_check_overflow_instructions(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
    ) {
        let tmp = module.locals.add(walrus::ValType::I32);
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

    /// Adds the instructions to add two u8 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than 255
    /// then the execution is aborted This check is poosible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        builder.binop(BinaryOp::I32Add);
        Self::add_check_overflow_instructions(builder, module);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        memory: MemoryId,
    ) {
        match original_type {
            IntermediateType::IU8 => {}
            IntermediateType::IU16 | IntermediateType::IU32 => {
                // Just check for overflow and leave the value in the stack again
                Self::add_check_overflow_instructions(builder, module);
            }
            IntermediateType::IU64 => {
                builder.unop(UnaryOp::I32WrapI64);
                Self::add_check_overflow_instructions(builder, module);
            }
            IntermediateType::IU128 => {
                downcast_u128_to_u32(builder, module, memory);
                Self::add_check_overflow_instructions(builder, module);
            }
            IntermediateType::IU256 => {
                downcast_u256_to_u32(builder, module, memory);
                Self::add_check_overflow_instructions(builder, module);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
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

    fn add_check_overflow_instructions(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
    ) {
        let tmp = module.locals.add(walrus::ValType::I32);
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

    /// Adds the instructions to add two u16 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 65535 then the execution is aborted. This check is poosible because interally we are using
    /// 32bits integers.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        builder.binop(BinaryOp::I32Add);
        Self::add_check_overflow_instructions(builder, module);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        memory: MemoryId,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 => {}
            IntermediateType::IU32 => {
                // Just check for overflow and leave the value in the stack again
                Self::add_check_overflow_instructions(builder, module);
            }
            IntermediateType::IU64 => {
                builder.unop(UnaryOp::I32WrapI64);
                Self::add_check_overflow_instructions(builder, module);
            }
            IntermediateType::IU128 => {
                downcast_u128_to_u32(builder, module, memory);
                Self::add_check_overflow_instructions(builder, module);
            }
            IntermediateType::IU256 => {
                downcast_u256_to_u32(builder, module, memory);
                Self::add_check_overflow_instructions(builder, module);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct IU32;

impl IU32 {
    const MAX_VALUE: i64 = u32::MAX as i64;

    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(4).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }

    /// Adds the instructions to add two u32 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 4_294_967_295 then the execution is aborted. To check the overflow we check that the result
    /// is strictly greater than the two operands. Because we are using i32 integer, if the
    /// addition overflow, WASM wraps around the result.
    ///
    /// NOTE: We use two temporal local variables to do the checks (n1, n2). If a program contains
    /// a lot of additions we will add two local variables per addition. We can optimize this by
    /// tracking and reuse the same ones used in the first addition found.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let res = module.locals.add(ValType::I32);
        let n1 = module.locals.add(ValType::I32);
        let n2 = module.locals.add(ValType::I32);

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

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        memory: MemoryId,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {}
            IntermediateType::IU64 => {
                // Check first that the i64 fits in an i32
                let tmp = module.locals.add(walrus::ValType::I64);
                builder.local_tee(tmp);
                builder.i64_const(Self::MAX_VALUE);
                builder.binop(BinaryOp::I64GtU);
                builder.if_else(
                    Some(ValType::I64),
                    |then| {
                        then.unreachable();
                    },
                    |else_| {
                        else_.local_get(tmp);
                    },
                );

                builder.unop(UnaryOp::I32WrapI64);
            }
            IntermediateType::IU128 => {
                downcast_u128_to_u32(builder, module, memory);
            }
            IntermediateType::IU256 => {
                downcast_u256_to_u32(builder, module, memory);
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

    /// Adds the instructions to add two u64 values.
    ///
    /// Along with the addition code to check overflow is added. If the result is greater than
    /// 18_446_744_073_709_551_615 then the execution is aborted. To check the overflow we check
    /// that the result is strictly greater than the two operands. Because we are using i32
    /// integer, if the addition overflow, WASM wraps around the result.
    ///
    /// NOTE: We use two temporal local variables to do the checks (n1, n2). If a program contains
    /// a lot of additions we will add two local variables per addition. We can optimize this by
    /// tracking and reuse the same ones used in the first addition found.
    pub fn add(builder: &mut walrus::InstrSeqBuilder, module: &mut walrus::Module) {
        let res = module.locals.add(ValType::I64);
        let n1 = module.locals.add(ValType::I64);
        let n2 = module.locals.add(ValType::I64);

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

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        memory: MemoryId,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                builder.unop(UnaryOp::I64ExtendUI32);
            }
            IntermediateType::IU64 => {}
            IntermediateType::IU128 => {
                let reader_pointer = module.locals.add(ValType::I32);
                builder.local_tee(reader_pointer);
                builder.load(
                    memory,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // Ensure the rest bytes are zero, otherwise would have overflowed
                builder.block(None, |inner_block| {
                    let inner_block_id = inner_block.id();

                    inner_block.local_get(reader_pointer);
                    inner_block.load(
                        memory,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 8,
                        },
                    );
                    inner_block.i64_const(0);
                    inner_block.binop(BinaryOp::I64Eq);
                    inner_block.br_if(inner_block_id);
                    inner_block.unreachable();
                });
            }
            IntermediateType::IU256 => {
                let reader_pointer = module.locals.add(ValType::I32);
                builder.local_tee(reader_pointer);
                builder.load(
                    memory,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // Ensure the rest bytes are zero, otherwise would have overflowed
                for i in 0..3 {
                    builder.block(None, |inner_block| {
                        let inner_block_id = inner_block.id();

                        inner_block.local_get(reader_pointer);
                        inner_block.load(
                            memory,
                            LoadKind::I64 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 8 + i * 8,
                            },
                        );
                        inner_block.i64_const(0);
                        inner_block.binop(BinaryOp::I64Eq);
                        inner_block.br_if(inner_block_id);
                        inner_block.unreachable();
                    });
                }
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }
}

fn downcast_u128_to_u32(
    builder: &mut walrus::InstrSeqBuilder,
    module: &mut walrus::Module,
    memory: MemoryId,
) {
    let reader_pointer = module.locals.add(ValType::I32);
    builder.local_tee(reader_pointer);
    builder.load(
        memory,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Ensure the rest bytes are zero, otherwise would have overflowed
    for i in 0..3 {
        builder.block(None, |inner_block| {
            let inner_block_id = inner_block.id();

            inner_block.local_get(reader_pointer);
            inner_block.load(
                memory,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4 + i * 4,
                },
            );
            inner_block.i32_const(0);
            inner_block.binop(BinaryOp::I32Eq);
            inner_block.br_if(inner_block_id);
            inner_block.unreachable();
        });
    }
}

fn downcast_u256_to_u32(
    builder: &mut walrus::InstrSeqBuilder,
    module: &mut walrus::Module,
    memory: MemoryId,
) {
    let reader_pointer = module.locals.add(ValType::I32);
    builder.local_tee(reader_pointer);
    builder.load(
        memory,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Ensure the rest bytes are zero, otherwise would have overflowed
    for i in 0..7 {
        builder.block(None, |inner_block| {
            let inner_block_id = inner_block.id();

            inner_block.local_get(reader_pointer);
            inner_block.load(
                memory,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4 + i * 4,
                },
            );
            inner_block.i32_const(0);
            inner_block.binop(BinaryOp::I32Eq);
            inner_block.br_if(inner_block_id);
            inner_block.unreachable();
        });
    }
}

use walrus::{
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::IntermediateType;

#[derive(Clone, Copy)]
pub struct IU128;

impl IU128 {
    pub fn load_constant_instructions(
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 16] = bytes.take(16).collect::<Vec<u8>>().try_into().unwrap();

        let pointer = module_locals.add(ValType::I32);

        builder.i32_const(bytes.len() as i32);
        builder.call(allocator);
        builder.local_set(pointer);

        let mut offset = 0;

        while offset < bytes.len() {
            builder.local_get(pointer);
            builder.i64_const(i64::from_le_bytes(
                bytes[offset..offset + 8].try_into().unwrap(),
            ));
            builder.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: offset as u32,
                },
            );

            offset += 8;
        }

        builder.local_get(pointer);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module_locals: &mut walrus::ModuleLocals,
        original_type: IntermediateType,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                let value_local = module_locals.add(ValType::I32);
                builder.local_set(value_local);

                let pointer = module_locals.add(ValType::I32);

                builder.i32_const(16);
                builder.call(allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    memory,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU64 => {
                let value_local = module_locals.add(ValType::I64);
                builder.local_set(value_local);

                let pointer = module_locals.add(ValType::I32);

                builder.i32_const(16);
                builder.call(allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    memory,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU128 => {}
            IntermediateType::IU256 => {
                let original_pointer = module_locals.add(ValType::I32);
                builder.local_set(original_pointer);

                let pointer = module_locals.add(ValType::I32);

                builder.i32_const(16);
                builder.call(allocator);
                builder.local_set(pointer);

                for i in 0..2 {
                    builder.local_get(pointer);
                    builder.local_get(original_pointer);
                    builder.load(
                        memory,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: i * 8,
                        },
                    );
                    builder.store(
                        memory,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: i * 8,
                        },
                    );
                }

                // Ensure the rest bytes are zero, otherwise it would have overflowed
                for i in 0..2 {
                    builder.block(None, |inner_block| {
                        let inner_block_id = inner_block.id();

                        inner_block.local_get(pointer);
                        inner_block.load(
                            memory,
                            LoadKind::I64 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 16 + i * 8,
                            },
                        );
                        inner_block.i64_const(0);
                        inner_block.binop(BinaryOp::I64Eq);
                        inner_block.br_if(inner_block_id);
                        inner_block.unreachable();
                    });
                }
                builder.local_get(pointer);
            }
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct IU256;

impl IU256 {
    pub fn load_constant_instructions(
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 32] = bytes.take(32).collect::<Vec<u8>>().try_into().unwrap();

        let pointer = module_locals.add(ValType::I32);

        builder.i32_const(bytes.len() as i32);
        builder.call(allocator);
        builder.local_set(pointer);

        let mut offset = 0;

        while offset < bytes.len() {
            builder.local_get(pointer);
            builder.i64_const(i64::from_le_bytes(
                bytes[offset..offset + 8].try_into().unwrap(),
            ));
            builder.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: offset as u32,
                },
            );

            offset += 8;
        }

        builder.local_get(pointer);
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module_locals: &mut walrus::ModuleLocals,
        original_type: IntermediateType,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                let value_local = module_locals.add(ValType::I32);
                builder.local_set(value_local);

                let pointer = module_locals.add(ValType::I32);

                builder.i32_const(32);
                builder.call(allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    memory,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU64 => {
                let value_local = module_locals.add(ValType::I64);
                builder.local_set(value_local);

                let pointer = module_locals.add(ValType::I32);

                builder.i32_const(32);
                builder.call(allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    memory,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU128 => {
                let original_pointer = module_locals.add(ValType::I32);
                builder.local_set(original_pointer);

                let pointer = module_locals.add(ValType::I32);

                builder.i32_const(32);
                builder.call(allocator);
                builder.local_set(pointer);

                for i in 0..2 {
                    builder.local_get(pointer);
                    builder.local_get(original_pointer);
                    builder.load(
                        memory,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: i * 8,
                        },
                    );
                    builder.store(
                        memory,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: i * 8,
                        },
                    );
                }

                builder.local_get(pointer);
            }
            IntermediateType::IU256 => {}
            t => panic!("type stack error: trying to cast {t:?}"),
        }
    }
}

use walrus::{
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use super::IntermediateType;

fn add(
    builder: &mut walrus::InstrSeqBuilder,
    module_locals: &mut walrus::ModuleLocals,
    memory: MemoryId,
    allocator: FunctionId,
    type_heap_size: i32,
) {
    let pointer = module_locals.add(ValType::I32);
    let offset = module_locals.add(ValType::I32);
    let overflowed = module_locals.add(ValType::I32);
    let partial_sum = module_locals.add(ValType::I64);
    let n1_ptr = module_locals.add(ValType::I32);
    let n2_ptr = module_locals.add(ValType::I32);
    let n1 = module_locals.add(ValType::I64);
    let n2 = module_locals.add(ValType::I64);

    // Allocate memory for the result
    builder
        // Save the pointers of the numbers to be added
        .local_set(n1_ptr)
        .local_set(n2_ptr)
        // Allocate memory for the result
        .i32_const(type_heap_size)
        .call(allocator)
        .local_set(pointer)
        // Set the offset to 0
        .i32_const(0)
        .local_set(offset)
        // Set the overflowed to false
        .i32_const(0)
        .local_set(overflowed);

    builder
        .block(None, |block| {
            let block_id = block.id();
            block.loop_(None, |loop_| {
                let loop_id = loop_.id();
                // Load a part of the first operand and save it in n1
                loop_
                    // Read the first operand
                    .local_get(n1_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        memory,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(n1)
                    // Read the second operand
                    .local_get(n2_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        memory,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(n2)
                    // We add the two loaded parts
                    .binop(BinaryOp::I64Add)
                    // And add the rest of the previous operation
                    // Here we use the fact that the rest is always 1 and that the overflowed flag
                    // is either 1 if there was an overflow or 0 if not. If there was an overflow
                    // we need to add 1 to the sum so, we re-use the variable
                    .local_get(overflowed)
                    .unop(UnaryOp::I64ExtendUI32)
                    .binop(BinaryOp::I64Add)
                    // Save the result to partial_sum
                    .local_set(partial_sum);

                // Save chunk of 64 bits
                loop_
                    .local_get(pointer)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .local_get(partial_sum)
                    .store(
                        memory,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // Check overflow
                loop_
                    // If either n1 and n2 is zero or rest is not zero then there can be an overflow
                    // (n1 != 0) && (n2 != 0) || (rest != 0)
                    // where rest = overflowed
                    .local_get(n1)
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .local_get(n2)
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .binop(BinaryOp::I32And)
                    .local_get(overflowed)
                    .unop(UnaryOp::I64ExtendUI32)
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .binop(BinaryOp::I32Or);

                // If partial sum is less or equal than any of the sumands then an overflow ocurred
                // (partial_sum <= n1) || (partial_sum <= n2)
                loop_
                    .local_get(partial_sum)
                    .local_get(n1)
                    .binop(BinaryOp::I64LeU)
                    .local_get(partial_sum)
                    .local_get(n2)
                    .binop(BinaryOp::I64LeU)
                    .binop(BinaryOp::I32Or)
                    // If the following condition is true, there was overflow
                    // ((n1 != 0) && (n2 != 0) || (rest != 0)) && ((partial_sum <= n1) || (partial_sum <= n2))
                    .binop(BinaryOp::I32And)
                    .local_set(overflowed);

                // We check if we are adding the last chunks of the operands
                loop_
                    .local_get(offset)
                    .i32_const(type_heap_size - 8)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        None,
                        |then| {
                            // If an overflow happened in the last chunk, means the whole number
                            // overflowed
                            then.local_get(overflowed).if_else(
                                None,
                                |then| {
                                    then.unreachable();
                                },
                                // Otherwise we finished the  addition
                                |else_| {
                                    else_.br(block_id);
                                },
                            );
                        },
                        // If we are not in the last chunk, we continue the iteration
                        |else_| {
                            // offset += 8 and process the next part of the integer
                            else_
                                .local_get(offset)
                                .i32_const(8)
                                .binop(BinaryOp::I32Add)
                                .local_set(offset)
                                .br(loop_id);
                        },
                    );
            });
        })
        // Return the address of the sum
        .local_get(pointer);
}

fn compare_heap_integers_bitwise(
    builder: &mut walrus::InstrSeqBuilder,
    module_locals: &mut walrus::ModuleLocals,
    memory: MemoryId,
    allocator: FunctionId,
    heap_size: i32,
    comparator: BinaryOp,
) {
    let num_1 = module_locals.add(ValType::I32);
    let num_2 = module_locals.add(ValType::I32);
    builder.local_set(num_2).local_set(num_1);

    let pointer = module_locals.add(ValType::I32);

    builder
        .i32_const(heap_size)
        .call(allocator)
        .local_set(pointer);

    let pages = heap_size as u32 / 8;
    for i in 0..pages {
        builder.local_get(pointer);

        builder
            .local_get(num_1)
            .load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            )
            .local_get(num_2)
            .load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );

        builder.binop(comparator).store(
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

#[derive(Clone, Copy)]
pub struct IU128;

impl IU128 {
    /// Heap size (in bytes)
    pub const HEAP_SIZE: i32 = 16;

    pub fn load_constant_instructions(
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; Self::HEAP_SIZE as usize] = bytes
            .take(Self::HEAP_SIZE as usize)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

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

    pub fn bit_or(
        builder: &mut InstrSeqBuilder,
        module_locals: &mut ModuleLocals,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module_locals,
            memory,
            allocator,
            Self::HEAP_SIZE,
            BinaryOp::I64Or,
        );
    }

    pub fn bit_and(
        builder: &mut InstrSeqBuilder,
        module_locals: &mut ModuleLocals,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module_locals,
            memory,
            allocator,
            Self::HEAP_SIZE,
            BinaryOp::I64And,
        );
    }

    pub fn bit_xor(
        builder: &mut InstrSeqBuilder,
        module_locals: &mut ModuleLocals,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module_locals,
            memory,
            allocator,
            Self::HEAP_SIZE,
            BinaryOp::I64Xor,
        );
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

    pub fn add(
        builder: &mut walrus::InstrSeqBuilder,
        module_locals: &mut walrus::ModuleLocals,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        add(builder, module_locals, memory, allocator, Self::HEAP_SIZE);
    }
}

#[derive(Clone, Copy)]
pub struct IU256;

impl IU256 {
    /// Heap size (in bytes)
    pub const HEAP_SIZE: i32 = 32;

    pub fn load_constant_instructions(
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; Self::HEAP_SIZE as usize] = bytes
            .take(Self::HEAP_SIZE as usize)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

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

    pub fn bit_or(
        builder: &mut InstrSeqBuilder,
        module_locals: &mut ModuleLocals,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module_locals,
            memory,
            allocator,
            Self::HEAP_SIZE,
            BinaryOp::I64Or,
        );
    }

    pub fn bit_and(
        builder: &mut InstrSeqBuilder,
        module_locals: &mut ModuleLocals,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module_locals,
            memory,
            allocator,
            Self::HEAP_SIZE,
            BinaryOp::I64And,
        );
    }

    pub fn bit_xor(
        builder: &mut InstrSeqBuilder,
        module_locals: &mut ModuleLocals,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module_locals,
            memory,
            allocator,
            Self::HEAP_SIZE,
            BinaryOp::I64Xor,
        );
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

    pub fn add(
        builder: &mut walrus::InstrSeqBuilder,
        module_locals: &mut walrus::ModuleLocals,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        add(builder, module_locals, memory, allocator, Self::HEAP_SIZE);
    }
}

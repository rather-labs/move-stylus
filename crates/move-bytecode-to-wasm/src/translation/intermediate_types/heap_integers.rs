use walrus::{
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
};

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
        let bytes: [u8; Self::HEAP_SIZE as usize] =
            bytes.take(16).collect::<Vec<u8>>().try_into().unwrap();

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

    pub fn add(
        builder: &mut walrus::InstrSeqBuilder,
        module_locals: &mut walrus::ModuleLocals,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        let pointer = module_locals.add(ValType::I32);
        let offset = module_locals.add(ValType::I32);
        // This can contain the sum or the partial_sum, depends if it overflows or not
        let partial_sum = module_locals.add(ValType::I64);
        let rest = module_locals.add(ValType::I64);
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
            .i32_const(Self::HEAP_SIZE)
            .call(allocator)
            .local_set(pointer)
            // Set the rest to 0
            .i64_const(0)
            .local_set(rest)
            // Set the offset to 0
            .i32_const(0)
            .local_set(offset);

        builder
            .block(None, |block| {
                let block_id = block.id();
                block.loop_(None, |loop_| {
                    let loop_id = loop_.id();

                    loop_
                        // Load a part of the first operand and save it in n1
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
                        .local_set(n1)
                        // Load a part of the second operand and save it in n2
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
                        .local_get(n1)
                        // We add the two loaded parts
                        .binop(BinaryOp::I64Add)
                        // And add the rest of the previous operation (if there was none, its the
                        // rest is 0)
                        .local_get(rest)
                        // .binop(BinaryOp::I64Xor)
                        .binop(BinaryOp::I64Add)
                        // Save the result to partial_sum and check if overflow ocurred
                        .local_tee(partial_sum)
                        .local_get(n1)
                        .binop(BinaryOp::I64LtU)
                        .local_get(partial_sum)
                        .local_get(n2)
                        .binop(BinaryOp::I64LtU)
                        .binop(BinaryOp::I32Or)
                        // If overflow ocurred
                        .if_else(
                            None,
                            |then| {
                                // If we are in overflow and the offset is 16, means the whole
                                // number overflowed and we are out of space
                                then.local_get(offset)
                                    .i32_const(16)
                                    .binop(BinaryOp::I32Eq)
                                    .if_else(
                                        None,
                                        |then| {
                                            then.unreachable();
                                        },
                                        |else_| {
                                            else_
                                                // We store in ponter + offset
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
                                                )
                                                .local_get(partial_sum)
                                                // The rest is the compliment of the the sum
                                                .i64_const(-1)
                                                .binop(BinaryOp::I64Xor)
                                                .local_set(rest)
                                                // offset += 8 and process the next part of the integer
                                                .local_get(offset)
                                                .i32_const(8)
                                                .binop(BinaryOp::I32Add)
                                                .local_set(offset)
                                                .br(loop_id);
                                        },
                                    );
                            },
                            // If we are not in overflow, we just save the partial_sum and return
                            |else_| {
                                else_
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
                                    )
                                    .br(block_id);
                            },
                        );
                });
            })
            .local_get(pointer);
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
}

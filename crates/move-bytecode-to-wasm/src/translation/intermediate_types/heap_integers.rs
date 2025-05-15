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
        // This can contain the sum or the rest, depends if it overflows or not
        let rest = module_locals.add(ValType::I64);
        let n1_ptr = module_locals.add(ValType::I32);
        let n2_ptr = module_locals.add(ValType::I32);
        let n1 = module_locals.add(ValType::I64);
        let n2 = module_locals.add(ValType::I64);

        // Allocate memory for the result
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(allocator)
            .local_set(pointer)
            // Save the pointers of the numbers to be added
            .local_set(n1_ptr)
            .local_set(n2_ptr)
            // Set the rest to 0
            .i64_const(0)
            .local_set(rest)
            // Set the offset to 8 (last 64 bits of u128)
            .i32_const(8)
            .local_set(offset);

        // Proably the type is i32
        builder
            .block(None, |block| {
                let block_id = block.id();
                block.loop_(None, |loop_| {
                    let loop_id = loop_.id();

                    loop_
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
                        .binop(BinaryOp::I64Add)
                        // Add the rest of the previous operation (if there was none, its the rest is
                        // initialized as 0)
                        .local_get(rest)
                        .binop(BinaryOp::I64Add)
                        // Check if overflow ocurred
                        .local_tee(rest)
                        .local_get(n1)
                        .binop(BinaryOp::I64LtU)
                        .local_get(rest)
                        .local_get(n2)
                        .binop(BinaryOp::I64LtU)
                        .binop(BinaryOp::I32Or)
                        // If overflow ocurred
                        .if_else(
                            None,
                            |then| {
                                // If we are in overflow and the offset is 0, means the whole
                                // number overflowed and we are out of space
                                then.local_get(offset)
                                    .i32_const(0)
                                    .binop(BinaryOp::I32Eq)
                                    .if_else(
                                        None,
                                        |then| {
                                            then.unreachable();
                                        },
                                        // Otherwise Set this part of the memory as all 1 (-1 =
                                        // 0xFFFFFFFFFFFFFFFFF because of two compliment)
                                        |else_| {
                                            else_
                                                // We store in ponter - offset
                                                .local_get(pointer)
                                                .local_get(offset)
                                                .binop(BinaryOp::I32Sub)
                                                .i64_const(-1)
                                                .store(
                                                    memory,
                                                    StoreKind::I64 { atomic: false },
                                                    MemArg {
                                                        align: 0,
                                                        offset: 0,
                                                    },
                                                )
                                                // offset -= 8 and process the next part of the integer
                                                .local_get(offset)
                                                .i32_const(8)
                                                .binop(BinaryOp::I32Sub)
                                                .br(loop_id);
                                        },
                                    );
                            },
                            // If we are not in overflow, we just save the rest and return
                            |else_| {
                                else_
                                    .local_get(pointer)
                                    .local_get(rest)
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

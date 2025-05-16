use walrus::{
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
};

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
    let rest = module_locals.add(ValType::I64);
    let n1_ptr = module_locals.add(ValType::I32);
    let n2_ptr = module_locals.add(ValType::I32);
    let n1 = module_locals.add(ValType::I32);
    let n2 = module_locals.add(ValType::I32);

    // Allocate memory for the result
    builder
        // Save the pointers of the numbers to be added
        .local_set(n1_ptr)
        .local_set(n2_ptr)
        // Allocate memory for the result
        .i32_const(type_heap_size)
        .call(allocator)
        .local_set(pointer)
        // Set the rest to 0
        .i64_const(0)
        .local_set(rest)
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
                // ===
                // Read the first 32 bits of the two operands and add them in a 64 bit int
                // ==
                // Load a part of the first operand and save it in n1
                loop_
                    // Read the first 32 bits of n1 and put then in a 64 bits integer
                    .local_get(n1_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        memory,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(n1)
                    .unop(UnaryOp::I64ExtendUI32)
                    // Read the first 32 bits of n2 and put then in a 64 bits integer
                    .local_get(n2_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        memory,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(n2)
                    .unop(UnaryOp::I64ExtendUI32)
                    // We add the two loaded parts
                    .binop(BinaryOp::I64Add)
                    // And add the rest of the previous operation (if there was none, its the
                    // rest is 0)
                    .local_get(rest)
                    .binop(BinaryOp::I64Add)
                    // Save the result to partial_sum
                    .local_set(partial_sum);

                // Save the lower 32 bits of the result
                loop_
                    .local_get(pointer)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .local_get(partial_sum)
                    // TODO I think this wraps, we need to mask the rest part
                    // to zero so we avoid wrapping
                    .unop(UnaryOp::I32WrapI64)
                    .store(
                        memory,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // ===
                // After the addition, the lower 32 bits are the result we are going to store and the higher 32
                // bits are carried to sum in the next iteration
                // ==
                loop_
                    .local_get(partial_sum)
                    .i64_const(32)
                    .binop(BinaryOp::I64ShrU)
                    .local_tee(rest);

                // Set if there is overflow
                loop_
                    .i64_const(0)
                    .binop(BinaryOp::I64Ne)
                    .local_set(overflowed);

                // We check if we are the offset is out of bounds
                loop_
                    .local_get(offset)
                    .i32_const(type_heap_size - 4)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        None,
                        |then| {
                            // If we are out of bound we check if overflow ocurred, if that
                            // happened then we trap
                            // Check if overflow ocurred
                            then.local_get(overflowed).if_else(
                                None,
                                |then| {
                                    then.unreachable();
                                },
                                |else_| {
                                    else_.br(block_id);
                                },
                            );
                        },
                        // Otherwise we make store the result and recalculate the rest
                        |else_| {
                            // offset += 4 and process the next part of the integer
                            else_
                                .local_get(offset)
                                .i32_const(4)
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

    pub fn add(
        builder: &mut walrus::InstrSeqBuilder,
        module_locals: &mut walrus::ModuleLocals,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        add(builder, module_locals, memory, allocator, Self::HEAP_SIZE);
    }
}

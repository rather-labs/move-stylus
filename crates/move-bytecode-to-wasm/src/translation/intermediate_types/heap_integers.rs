use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, runtime::RuntimeFunction};

use super::IntermediateType;

fn compare_heap_integers_bitwise(
    builder: &mut walrus::InstrSeqBuilder,
    module: &mut walrus::Module,
    compilation_ctx: &CompilationContext,
    heap_size: i32,
    comparator: BinaryOp,
) {
    let num_1 = module.locals.add(ValType::I32);
    let num_2 = module.locals.add(ValType::I32);
    builder.local_set(num_2).local_set(num_1);

    let pointer = module.locals.add(ValType::I32);

    builder
        .i32_const(heap_size)
        .call(compilation_ctx.allocator)
        .local_set(pointer);

    let pages = heap_size as u32 / 8;
    for i in 0..pages {
        builder.local_get(pointer);

        builder
            .local_get(num_1)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            )
            .local_get(num_2)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );

        builder.binop(comparator).store(
            compilation_ctx.memory_id,
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
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        compilation_ctx: &CompilationContext,
    ) {
        let bytes: [u8; Self::HEAP_SIZE as usize] = bytes
            .take(Self::HEAP_SIZE as usize)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        let pointer = module.locals.add(ValType::I32);

        builder.i32_const(bytes.len() as i32);
        builder.call(compilation_ctx.allocator);
        builder.local_set(pointer);

        let mut offset = 0;

        while offset < bytes.len() {
            builder.local_get(pointer);
            builder.i64_const(i64::from_le_bytes(
                bytes[offset..offset + 8].try_into().unwrap(),
            ));
            builder.store(
                compilation_ctx.memory_id,
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
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module,
            compilation_ctx,
            Self::HEAP_SIZE,
            BinaryOp::I64Or,
        );
    }

    pub fn bit_and(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module,
            compilation_ctx,
            Self::HEAP_SIZE,
            BinaryOp::I64And,
        );
    }

    pub fn bit_xor(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module,
            compilation_ctx,
            Self::HEAP_SIZE,
            BinaryOp::I64Xor,
        );
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                let value_local = module.locals.add(ValType::I32);
                builder.local_set(value_local);

                let pointer = module.locals.add(ValType::I32);

                builder.i32_const(16);
                builder.call(compilation_ctx.allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU64 => {
                let value_local = module.locals.add(ValType::I64);
                builder.local_set(value_local);

                let pointer = module.locals.add(ValType::I32);

                builder.i32_const(16);
                builder.call(compilation_ctx.allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    compilation_ctx.memory_id,
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
                let original_pointer = module.locals.add(ValType::I32);
                builder.local_set(original_pointer);

                let pointer = module.locals.add(ValType::I32);

                builder.i32_const(16);
                builder.call(compilation_ctx.allocator);
                builder.local_set(pointer);

                for i in 0..2 {
                    builder.local_get(pointer);
                    builder.local_get(original_pointer);
                    builder.load(
                        compilation_ctx.memory_id,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: i * 8,
                        },
                    );
                    builder.store(
                        compilation_ctx.memory_id,
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
                            compilation_ctx.memory_id,
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
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let add_function_id = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));
        // Alocate space for the result
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(compilation_ctx.allocator);
        builder.i32_const(Self::HEAP_SIZE).call(add_function_id);
    }

    pub fn bit_shift_left(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let shift_left_function_id =
            RuntimeFunction::HeapIntShiftLeft.get(module, Some(compilation_ctx));
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(shift_left_function_id);
    }

    pub fn bit_shift_right(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let shift_right_function_id =
            RuntimeFunction::HeapIntShiftRight.get(module, Some(compilation_ctx));
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(shift_right_function_id);
    }

    pub fn sub(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let sub_function_id = RuntimeFunction::HeapIntSub.get(module, Some(compilation_ctx));
        // Alocate space for the result
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(compilation_ctx.allocator);
        builder.i32_const(Self::HEAP_SIZE).call(sub_function_id);
    }

    pub fn mul(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let mul_function_id = RuntimeFunction::HeapIntMul.get(module, Some(compilation_ctx));

        builder.i32_const(Self::HEAP_SIZE).call(mul_function_id);
    }

    pub fn div(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let div_mod_function_id = RuntimeFunction::HeapIntDivMod.get(module, Some(compilation_ctx));

        builder
            .i32_const(Self::HEAP_SIZE)
            .i32_const(1)
            .call(div_mod_function_id);
    }

    pub fn remainder(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let div_mod_function_id = RuntimeFunction::HeapIntDivMod.get(module, Some(compilation_ctx));

        builder
            .i32_const(Self::HEAP_SIZE)
            .i32_const(0)
            .call(div_mod_function_id);
    }

    pub fn equality(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let equality_f_id = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
        builder.i32_const(Self::HEAP_SIZE).call(equality_f_id);
    }
}

#[derive(Clone, Copy)]
pub struct IU256;

impl IU256 {
    /// Heap size (in bytes)
    pub const HEAP_SIZE: i32 = 32;

    pub fn load_constant_instructions(
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        compilation_ctx: &CompilationContext,
    ) {
        let bytes: [u8; Self::HEAP_SIZE as usize] = bytes
            .take(Self::HEAP_SIZE as usize)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        let pointer = module.locals.add(ValType::I32);

        builder.i32_const(bytes.len() as i32);
        builder.call(compilation_ctx.allocator);
        builder.local_set(pointer);

        let mut offset = 0;

        while offset < bytes.len() {
            builder.local_get(pointer);
            builder.i64_const(i64::from_le_bytes(
                bytes[offset..offset + 8].try_into().unwrap(),
            ));
            builder.store(
                compilation_ctx.memory_id,
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
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module,
            compilation_ctx,
            Self::HEAP_SIZE,
            BinaryOp::I64Or,
        );
    }

    pub fn bit_and(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module,
            compilation_ctx,
            Self::HEAP_SIZE,
            BinaryOp::I64And,
        );
    }

    pub fn bit_xor(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        compare_heap_integers_bitwise(
            builder,
            module,
            compilation_ctx,
            Self::HEAP_SIZE,
            BinaryOp::I64Xor,
        );
    }

    pub fn cast_from(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        original_type: IntermediateType,
        compilation_ctx: &CompilationContext,
    ) {
        match original_type {
            IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                let value_local = module.locals.add(ValType::I32);
                builder.local_set(value_local);

                let pointer = module.locals.add(ValType::I32);

                builder.i32_const(32);
                builder.call(compilation_ctx.allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU64 => {
                let value_local = module.locals.add(ValType::I64);
                builder.local_set(value_local);

                let pointer = module.locals.add(ValType::I32);

                builder.i32_const(32);
                builder.call(compilation_ctx.allocator);
                builder.local_tee(pointer);

                builder.local_get(value_local);
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(pointer);
            }
            IntermediateType::IU128 => {
                let original_pointer = module.locals.add(ValType::I32);
                builder.local_set(original_pointer);

                let pointer = module.locals.add(ValType::I32);

                builder.i32_const(32);
                builder.call(compilation_ctx.allocator);
                builder.local_set(pointer);

                for i in 0..2 {
                    builder.local_get(pointer);
                    builder.local_get(original_pointer);
                    builder.load(
                        compilation_ctx.memory_id,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: i * 8,
                        },
                    );
                    builder.store(
                        compilation_ctx.memory_id,
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
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let add_function_id = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(compilation_ctx.allocator);
        builder.i32_const(Self::HEAP_SIZE).call(add_function_id);
    }

    pub fn bit_shift_left(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let shift_left_function_id =
            RuntimeFunction::HeapIntShiftLeft.get(module, Some(compilation_ctx));
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(shift_left_function_id);
    }

    pub fn bit_shift_right(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let shift_right_function_id =
            RuntimeFunction::HeapIntShiftRight.get(module, Some(compilation_ctx));
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(shift_right_function_id);
    }

    pub fn sub(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let sub_function_id = RuntimeFunction::HeapIntSub.get(module, Some(compilation_ctx));
        builder
            .i32_const(Self::HEAP_SIZE)
            .call(compilation_ctx.allocator);
        builder.i32_const(Self::HEAP_SIZE).call(sub_function_id);
    }

    pub fn mul(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let mul_function_id = RuntimeFunction::HeapIntMul.get(module, Some(compilation_ctx));
        builder.i32_const(Self::HEAP_SIZE).call(mul_function_id);
    }

    pub fn div(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let div_mod_function_id = RuntimeFunction::HeapIntDivMod.get(module, Some(compilation_ctx));

        builder
            .i32_const(Self::HEAP_SIZE)
            .i32_const(1)
            .call(div_mod_function_id);
    }

    pub fn remainder(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let div_mod_function_id = RuntimeFunction::HeapIntDivMod.get(module, Some(compilation_ctx));

        builder
            .i32_const(Self::HEAP_SIZE)
            .i32_const(0)
            .call(div_mod_function_id);
    }

    pub fn equality(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let equality_f_id = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
        builder.i32_const(Self::HEAP_SIZE).call(equality_f_id);
    }
}

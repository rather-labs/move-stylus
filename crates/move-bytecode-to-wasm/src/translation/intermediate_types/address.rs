use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::CompilationContext;

#[derive(Clone, Copy)]
pub struct IAddress;

impl IAddress {
    pub fn load_constant_instructions(
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        compilation_ctx: &CompilationContext,
    ) {
        let bytes: [u8; 32] = bytes.take(32).collect::<Vec<u8>>().try_into().unwrap();

        // Ensure the first 12 bytes are 0. Abi encoding restricts the address to be 20 bytes
        assert!(bytes[0..12].iter().all(|b| *b == 0));

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

    pub fn copy_local_instructions(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        let src_ptr = module.locals.add(ValType::I32);
        builder.local_set(src_ptr);

        builder.i32_const(32);
        builder.call(compilation_ctx.allocator);
        let dst_ptr = module.locals.add(ValType::I32);
        builder.local_set(dst_ptr);

        for i in 0..4 {
            builder.local_get(dst_ptr).local_get(src_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
        }
        for i in 0..4 {
            builder.store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 24 - i * 8,
                },
            );
        }
        builder.local_get(dst_ptr);
    }
}

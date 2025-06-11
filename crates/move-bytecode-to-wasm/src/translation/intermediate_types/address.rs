use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::{CompilationContext, runtime::RuntimeFunction};

#[derive(Clone, Copy)]
pub struct IAddress;

impl IAddress {
    /// Heap size (in bytes)
    pub const HEAP_SIZE: i32 = 32;

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

    pub fn equality(
        builder: &mut walrus::InstrSeqBuilder,
        module: &mut walrus::Module,
        compilation_ctx: &CompilationContext,
    ) {
        let equality_f_id = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
        builder.i32_const(Self::HEAP_SIZE).call(equality_f_id);
    }
}

use walrus::{
    FunctionId, InstrSeqBuilder, MemoryId, ModuleLocals, ValType,
    ir::{MemArg, StoreKind},
};

#[derive(Clone, Copy)]
pub struct IAddress;

impl IAddress {
    pub fn load_constant_instructions(
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 32] = bytes.take(32).collect::<Vec<u8>>().try_into().unwrap();

        // Ensure the first 12 bytes are 0. Abi encoding restricts the address to be 20 bytes
        assert!(bytes[0..12].iter().all(|b| *b == 0));

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

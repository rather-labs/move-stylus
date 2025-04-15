use walrus::{
    FunctionId, MemoryId, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::translation::functions::get_function_body_builder;

#[derive(Clone, Copy)]
pub struct IU128;

impl IU128 {
    pub fn load_constant_instructions(
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 16] = bytes.take(16).collect::<Vec<u8>>().try_into().unwrap();

        let pointer = module.locals.add(ValType::I32);

        let mut builder = get_function_body_builder(module, function_id);

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

#[derive(Clone, Copy)]
pub struct IU256;

impl IU256 {
    pub fn load_constant_instructions(
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 32] = bytes.take(32).collect::<Vec<u8>>().try_into().unwrap();

        let pointer = module.locals.add(ValType::I32);

        let mut builder = get_function_body_builder(module, function_id);

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

use walrus::{
    FunctionId, MemoryId, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::translation::functions::get_function_body_builder;

use super::{IParam, IntermediateType};

#[derive(Clone, Copy)]
pub struct IU128;

impl IParam for IU128 {}

impl IntermediateType for IU128 {
    fn to_wasm_type(&self) -> ValType {
        ValType::I32
    }

    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 16] = bytes.try_into().expect("Constant is not a u128");

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

impl IParam for IU256 {}

impl IntermediateType for IU256 {
    fn to_wasm_type(&self) -> ValType {
        ValType::I32
    }

    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let bytes: [u8; 32] = bytes.try_into().expect("Constant is not a u256");

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

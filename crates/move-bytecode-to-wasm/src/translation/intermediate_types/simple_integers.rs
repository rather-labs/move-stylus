use walrus::{FunctionId, MemoryId, Module};

use crate::wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions};

#[derive(Clone, Copy)]
pub struct IU8;

impl IU8 {
    pub fn load_constant_instructions(
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        let bytes = bytes.take(1).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(module, function_id, &bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU16;

impl IU16 {
    pub fn load_constant_instructions(
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        let bytes = bytes.take(2).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(module, function_id, &bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU32;

impl IU32 {
    pub fn load_constant_instructions(
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        let bytes = bytes.take(4).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(module, function_id, &bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU64;

impl IU64 {
    pub fn load_constant_instructions(
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        let bytes = bytes.take(8).collect::<Vec<u8>>();
        load_i64_from_bytes_instructions(module, function_id, &bytes);
    }
}

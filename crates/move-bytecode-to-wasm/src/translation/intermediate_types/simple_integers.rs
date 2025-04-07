use walrus::{FunctionId, MemoryId, Module, ValType};

use crate::wasm_helpers::{load_i32_from_bytes_instructions, load_i64_from_bytes_instructions};

use super::{IParam, IntermediateType};

#[derive(Clone, Copy)]
pub struct IU8;

impl IParam for IU8 {}

impl IntermediateType for IU8 {
    fn to_wasm_type(&self) -> ValType {
        ValType::I32
    }

    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        load_i32_from_bytes_instructions(module, function_id, bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU16;

impl IParam for IU16 {}

impl IntermediateType for IU16 {
    fn to_wasm_type(&self) -> ValType {
        ValType::I32
    }

    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        load_i32_from_bytes_instructions(module, function_id, bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU32;

impl IParam for IU32 {}

impl IntermediateType for IU32 {
    fn to_wasm_type(&self) -> ValType {
        ValType::I32
    }

    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        load_i32_from_bytes_instructions(module, function_id, bytes);
    }
}

#[derive(Clone, Copy)]
pub struct IU64;

impl IParam for IU64 {}

impl IntermediateType for IU64 {
    fn to_wasm_type(&self) -> ValType {
        ValType::I64
    }

    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        _allocator: FunctionId,
        _memory: MemoryId,
    ) {
        load_i64_from_bytes_instructions(module, function_id, bytes);
    }
}

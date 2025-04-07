use walrus::{FunctionId, MemoryId, Module, ValType};

use crate::wasm_helpers::load_i32_from_bytes_instructions;

use super::{IParam, IntermediateType};

#[derive(Clone, Copy)]
pub struct IBool;

impl IParam for IBool {}

impl IntermediateType for IBool {
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

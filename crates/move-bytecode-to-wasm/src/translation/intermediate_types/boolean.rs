use walrus::{FunctionId, MemoryId, Module};

use crate::wasm_helpers::load_i32_from_bytes_instructions;

#[derive(Clone, Copy)]
pub struct IBool;

impl IBool {
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

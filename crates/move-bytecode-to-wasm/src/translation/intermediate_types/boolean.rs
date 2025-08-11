use walrus::InstrSeqBuilder;

use crate::wasm_helpers::load_i32_from_bytes_instructions;

#[derive(Clone, Copy)]
pub struct IBool;

impl IBool {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) {
        let bytes = bytes.take(1).collect::<Vec<u8>>();
        load_i32_from_bytes_instructions(builder, &bytes);
    }
}

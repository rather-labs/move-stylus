use walrus::InstrSeqBuilder;

use crate::wasm_builder_extensions::WasmBuilderExtension;

use super::error::IntermediateTypeError;

#[derive(Clone, Copy)]
pub struct IBool;

impl IBool {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
    ) -> Result<(), IntermediateTypeError> {
        let bytes = bytes.take(1).collect::<Vec<u8>>();
        builder.load_i32_from_bytes(&bytes)?;
        Ok(())
    }
}

use walrus::InstrSeqBuilder;

use crate::wasm_builder_extensions::WasmBuilderExtension;

use super::error::IntermediateTypeError;

#[derive(Clone, Copy)]
pub struct IBool;

impl IBool {
    pub fn load_constant_instructions(
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::slice::Iter<'_, u8>,
    ) -> Result<(), IntermediateTypeError> {
        let bytes: [u8; 1] = std::array::from_fn(|_| bytes.next().copied().unwrap_or(0));
        builder.load_i32_from_bytes(&bytes)?;

        Ok(())
    }
}

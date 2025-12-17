use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext,
    compilation_context::CompilationContextError,
    compilation_context::ModuleId,
    compilation_context::reserved_modules::{SF_MODULE_NAME_BYTES, STYLUS_FRAMEWORK_ADDRESS},
};
use walrus::{InstrSeqBuilder, Module};

pub struct Bytes;

impl VmHandledType for Bytes {
    const IDENTIFIER: &str = "";

    fn inject(
        _block: &mut InstrSeqBuilder,
        _module: &mut Module,
        _compilation_ctx: &CompilationContext,
    ) {
        // no-op
    }

    fn is_vm_type(
        module_id: &ModuleId,
        index: u16,
        compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)?
            .identifier;

        // Check if identifier matches "Bytes" followed by a number 1-32
        if let Some(num_str) = identifier.strip_prefix("Bytes") {
            if let Ok(n) = num_str.parse::<u8>() {
                if (1..=32).contains(&n) {
                    if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                        || module_id.module_name != SF_MODULE_NAME_BYTES
                    {
                        return Err(VmHandledTypeError::InvalidFrameworkType(Box::leak(
                            identifier.clone().into_boxed_str(),
                        )));
                    }
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Bytes {
    // This should return an error if the identifier is not a valid BytesN type
    pub fn get_size_from_identifier(identifier: &str) -> Result<u8, CompilationContextError> {
        if let Some(num_str) = identifier.strip_prefix("Bytes") {
            if let Ok(n) = num_str.parse::<u8>() {
                return Ok(n);
            }
        }
        Err(CompilationContextError::InvalidBytesNIdentifier(
            identifier.to_string(),
        ))
    }
}

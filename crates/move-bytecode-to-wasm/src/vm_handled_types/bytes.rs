use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    compilation_context::reserved_modules::{SF_MODULE_NAME_SOL_TYPES, STYLUS_FRAMEWORK_ADDRESS},
};
use walrus::{InstrSeqBuilder, Module};

pub struct Bytes;

impl VmHandledType for Bytes {
    // Identifier varies with the size of the bytes struct, so it is not a constant
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
        // Check if the package is the stylus framework and the module is the bytes module
        if Bytes::validate_identifier(identifier) {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name != SF_MODULE_NAME_SOL_TYPES
            {
                return Err(VmHandledTypeError::InvalidFrameworkType(Box::leak(
                    identifier.clone().into_boxed_str(),
                )));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

impl Bytes {
    // Returns true if the identifier matches "BytesN" with N in 1..=32
    pub fn validate_identifier(identifier: &str) -> bool {
        if let Some(num_str) = identifier.strip_prefix("Bytes") {
            if let Ok(n) = num_str.parse::<u8>() {
                if (1..=32).contains(&n) {
                    return true;
                }
            }
        }
        false
    }
}

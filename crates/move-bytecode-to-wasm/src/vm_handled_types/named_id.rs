use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_OBJECT, STYLUS_FRAMEWORK_ADDRESS},
    },
};
use walrus::{InstrSeqBuilder, Module};

pub struct NamedId;

impl VmHandledType for NamedId {
    const IDENTIFIER: &str = "NamedId";

    fn inject(
        _block: &mut InstrSeqBuilder,
        _module: &mut Module,
        _compilation_ctx: &CompilationContext,
    ) {
        // UID is not injected, is created with a native function
    }

    fn is_vm_type(
        module_id: &ModuleId,
        index: u16,
        compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)?
            .identifier;

        if identifier == Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name.as_str() != SF_MODULE_NAME_OBJECT
            {
                return Err(VmHandledTypeError::InvalidFrameworkType(Self::IDENTIFIER));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

impl NamedId {
    const REMOVE_FN_IDENTIFIER: &str = "remove";

    pub fn is_remove_function(module_id: &ModuleId, identifier: &str) -> bool {
        identifier == Self::REMOVE_FN_IDENTIFIER
            && module_id.address == STYLUS_FRAMEWORK_ADDRESS
            && module_id.module_name.as_str() == SF_MODULE_NAME_OBJECT
    }
}

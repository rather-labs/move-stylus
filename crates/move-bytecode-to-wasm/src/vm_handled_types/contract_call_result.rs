use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_CONTRACT_CALLS, STYLUS_FRAMEWORK_ADDRESS},
    },
};
use walrus::{InstrSeqBuilder, Module};

pub struct ContractCallResult;

impl VmHandledType for ContractCallResult {
    const IDENTIFIER: &str = "ContractCallResult";

    fn inject(
        _block: &mut InstrSeqBuilder,
        _module: &mut Module,
        _compilation_ctx: &CompilationContext,
    ) {
        // Contract call result is not injected
    }

    fn is_vm_type(
        module_id: &ModuleId,
        index: u16,
        compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)?
            .identifier;

        if **identifier == *Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name.as_str() != SF_MODULE_NAME_CONTRACT_CALLS
            {
                return Err(VmHandledTypeError::InvalidFrameworkType(*identifier));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

pub struct ContractCallEmptyResult;

impl VmHandledType for ContractCallEmptyResult {
    const IDENTIFIER: &str = "ContractCallEmptyResult";

    fn inject(
        _block: &mut InstrSeqBuilder,
        _module: &mut Module,
        _compilation_ctx: &CompilationContext,
    ) {
        // Contract call result is not injected
    }

    fn is_vm_type(
        module_id: &ModuleId,
        index: u16,
        compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)?
            .identifier;

        if **identifier == *Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name.as_str() != SF_MODULE_NAME_CONTRACT_CALLS
            {
                return Err(VmHandledTypeError::InvalidFrameworkType(*identifier));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

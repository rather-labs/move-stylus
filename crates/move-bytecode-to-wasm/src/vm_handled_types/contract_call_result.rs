use super::VmHandledType;
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

    fn is_vm_type(module_id: &ModuleId, index: u16, compilation_ctx: &CompilationContext) -> bool {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)
            .unwrap()
            .identifier;

        if identifier == Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name != SF_MODULE_NAME_CONTRACT_CALLS
            {
                panic!(
                    "invalid ContractCallResult found, only the one from the stylus framework is valid"
                );
            }
            return true;
        }
        false
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

    fn is_vm_type(module_id: &ModuleId, index: u16, compilation_ctx: &CompilationContext) -> bool {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)
            .unwrap()
            .identifier;

        if identifier == Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name != SF_MODULE_NAME_CONTRACT_CALLS
            {
                panic!(
                    "invalid ContractCallEmptyResult found, only the one from the stylus framework is valid"
                );
            }
            return true;
        }
        false
    }
}

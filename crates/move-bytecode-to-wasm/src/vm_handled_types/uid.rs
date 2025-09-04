use super::VmHandledType;
use crate::{
    CompilationContext,
    compilation_context::{ModuleId, reserved_modules::STYLUS_FRAMEWORK_ADDRESS},
};
use walrus::{InstrSeqBuilder, Module};

pub struct Uid;

impl VmHandledType for Uid {
    const IDENTIFIER: &str = "UID";

    fn inject(
        _block: &mut InstrSeqBuilder,
        _module: &mut Module,
        _compilation_ctx: &CompilationContext,
    ) {
        // UID is not injected, is created with a native function
    }

    fn is_vm_type(module_id: &ModuleId, index: u16, compilation_ctx: &CompilationContext) -> bool {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)
            .unwrap()
            .identifier;

        if identifier == Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS || module_id.module_name != "object" {
                panic!("invalid UID found, only the one from the stylus framework is valid");
            }
            return true;
        }
        false
    }
}

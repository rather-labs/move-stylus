use super::VmHandledType;
use crate::{
    CompilationContext,
    compilation_context::{ModuleId, reserved_modules::STYLUS_FRAMEWORK_ADDRESS},
};
use walrus::{InstrSeqBuilder, Module};

pub struct TxContext;

impl VmHandledType for TxContext {
    const IDENTIFIER: &str = "TxContext";

    fn inject(
        block: &mut InstrSeqBuilder,
        _module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        block.i32_const(4).call(compilation_ctx.allocator);
    }

    fn is_vm_type(module_id: &ModuleId, identifier: &str) -> bool {
        if identifier == Self::IDENTIFIER {
            if module_id.address != STYLUS_FRAMEWORK_ADDRESS
                || module_id.module_name != "tx_context"
            {
                panic!("invalid TxContext found, only the one from the stylus framework is valid");
            }
            return true;
        }
        false
    }
}

use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_TX_CONTEXT, STYLUS_FRAMEWORK_ADDRESS},
    },
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
                || module_id.module_name.as_str() != SF_MODULE_NAME_TX_CONTEXT
            {
                return Err(VmHandledTypeError::InvalidFrameworkType(Self::IDENTIFIER));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

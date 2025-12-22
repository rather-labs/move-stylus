use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_FALLBACK, STYLUS_FRAMEWORK_ADDRESS},
    },
    data::DATA_CALLDATA_OFFSET,
};
use walrus::{
    InstrSeqBuilder, Module,
    ir::{LoadKind, MemArg},
};

pub struct Calldata;

impl VmHandledType for Calldata {
    const IDENTIFIER: &str = "Calldata";

    fn inject(
        block: &mut InstrSeqBuilder,
        _module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        block.i32_const(DATA_CALLDATA_OFFSET).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4,
            },
        );
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
                || module_id.module_name.as_str() != SF_MODULE_NAME_FALLBACK
            {
                return Err(VmHandledTypeError::InvalidFrameworkType(*identifier));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

use super::{VmHandledType, error::VmHandledTypeError};
use crate::{CompilationContext, compilation_context::ModuleId, runtime::RuntimeFunction};
use walrus::{InstrSeqBuilder, Module};

pub struct Signer;

impl VmHandledType for Signer {
    const IDENTIFIER: &str = "signer";

    fn inject(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        let inject_signer_fn = RuntimeFunction::InjectSigner
            .get(module, Some(compilation_ctx))
            .expect("failed to link inject_signer runtime function");
        block.call(inject_signer_fn);
    }

    fn is_vm_type(
        _module_id: &ModuleId,
        _index: u16,
        _compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        Ok(true)
    }
}

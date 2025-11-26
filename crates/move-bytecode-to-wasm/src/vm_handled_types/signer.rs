use super::{VmHandledType, error::VmHandledTypeError};
use crate::{
    CompilationContext, compilation_context::ModuleId, hostio::host_functions::tx_origin,
    translation::intermediate_types::signer::ISigner,
};
use walrus::{InstrSeqBuilder, Module, ValType, ir::BinaryOp};

pub struct Signer;

impl VmHandledType for Signer {
    const IDENTIFIER: &str = "signer";

    fn inject(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        let (tx_origin_function, _) = tx_origin(module);
        let signer_pointer = module.locals.add(ValType::I32);

        block
            .i32_const(ISigner::HEAP_SIZE)
            .call(compilation_ctx.allocator)
            .local_tee(signer_pointer);

        // We add 12 to the pointer returned by the allocator because stylus writes 20
        // bytes, and those bytes need to be at the end.
        block
            .i32_const(12)
            .binop(BinaryOp::I32Add)
            .call(tx_origin_function)
            .local_get(signer_pointer);
    }

    fn is_vm_type(
        _module_id: &ModuleId,
        _index: u16,
        _compilation_ctx: &CompilationContext,
    ) -> Result<bool, VmHandledTypeError> {
        Ok(true)
    }
}

use walrus::{InstrSeqBuilder, LocalId, Module};

use crate::{CompilationContext, translation::intermediate_types::structs::IStruct};

impl IStruct {
    pub fn add_pack_instructions(
        index: u16,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
    }
}

use walrus::{InstrSeqBuilder, LocalId, Module};

use crate::{CompilationContext, translation::intermediate_types::structs::IStruct};

impl IStruct {
    pub fn add_unpack_instructions(
        index: usize,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
    }
}

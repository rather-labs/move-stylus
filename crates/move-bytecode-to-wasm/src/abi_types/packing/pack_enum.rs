use walrus::{
    InstrSeqBuilder, LocalId, Module,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext, runtime::RuntimeFunction, translation::intermediate_types::enums::IEnum,
};

impl IEnum {
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        block.local_get(writer_pointer);

        // Little-endian to Big-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);

        // Read variant number
        block
            .local_get(local)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .call(swap_i32_bytes_function);

        block.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                // Abi is left-padded to 32 bytes
                offset: 28,
            },
        );
    }
}

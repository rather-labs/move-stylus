use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::CompilationContext;

/// Adds the instructions to load a literal heap type into memory.
///
/// It leaves a pointer to the value in the stack.
pub fn load_literal_heap_type_to_memory(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    bytes: &[u8],
) {
    let pointer = module.locals.add(ValType::I32);

    builder.i32_const(bytes.len() as i32);
    builder.call(compilation_ctx.allocator);
    builder.local_set(pointer);

    let mut offset = 0;

    while offset < bytes.len() {
        builder.local_get(pointer);
        builder.i64_const(i64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap(),
        ));
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: offset as u32,
            },
        );

        offset += 8;
    }

    builder.local_get(pointer);
}

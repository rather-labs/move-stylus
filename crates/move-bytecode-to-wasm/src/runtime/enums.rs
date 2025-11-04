use super::RuntimeFunction;
use crate::CompilationContext;
use crate::data::DATA_ENUM_STORAGE_SIZE_OFFSET;
use crate::translation::intermediate_types::IntermediateType;
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

/// Returns the storage size for an enum at a given slot offset (0-31).
///
/// Arguments:
/// - `slot_offset` (i32): byte offset (0-31) where the enum starts in the slot.
///
/// Returns:
/// - (i32): storage size in bytes for this enum at the given offset.
pub fn match_on_offset(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = RuntimeFunction::MatchOnOffset.get_generic_function_name(compilation_ctx, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();
    let enum_ = compilation_ctx
        .get_enum_by_intermediate_type(itype)
        .unwrap();

    // Argument
    let slot_offset = module.locals.add(ValType::I32);

    // Write storage_size vector to memory at DATA_ENUM_STORAGE_SIZE_OFFSET
    // Each value is stored as i32 at offset DATA_ENUM_STORAGE_SIZE_OFFSET + index * 4
    if let Some(storage_size) = &enum_.storage_size {
        for (index, &size) in storage_size.iter().enumerate() {
            builder
                .i32_const(DATA_ENUM_STORAGE_SIZE_OFFSET + (index as i32 * 4))
                .i32_const(size as i32)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
        }
    }

    // Load enum_size from memory using slot_offset as index
    builder
        .i32_const(DATA_ENUM_STORAGE_SIZE_OFFSET)
        .local_get(slot_offset)
        .i32_const(32)
        .binop(BinaryOp::I32RemU)
        .i32_const(4)
        .binop(BinaryOp::I32Mul)
        .binop(BinaryOp::I32Add)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function.finish(vec![slot_offset], &mut module.funcs)
}

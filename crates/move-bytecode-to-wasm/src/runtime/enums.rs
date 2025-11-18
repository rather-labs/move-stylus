use super::{RuntimeFunction, error::RuntimeFunctionError};
use crate::CompilationContext;
use crate::data::DATA_ENUM_STORAGE_SIZE_OFFSET;
use crate::translation::intermediate_types::IntermediateType;
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

const SLOT_SIZE: u32 = 32;

/// Returns the storage size for an enum at a given slot offset (0-31).
///
/// Arguments:
/// - `slot_offset` (i32): byte offset (0-31) where the enum starts in the slot.
///
/// Returns:
/// - (i32): storage size in bytes for this enum at the given offset.
pub fn get_storage_size_by_offset(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::GetStorageSizeByOffset
        .get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let enum_ = compilation_ctx
        .get_enum_by_intermediate_type(itype)
        .unwrap();

    // Calculate the enum storage sizes for each offset
    let storage_size = enum_.storage_size_by_offset(compilation_ctx).unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    // Argument
    let slot_offset = module.locals.add(ValType::I32);

    // Write storage_size vector to memory at DATA_ENUM_STORAGE_SIZE_OFFSET
    for (index, &size) in storage_size.iter().enumerate() {
        builder
            .i32_const(DATA_ENUM_STORAGE_SIZE_OFFSET)
            .i32_const(size as i32)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4 * index as u32,
                },
            );
    }

    // Load the enum storage size for the given offset from memory
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

    Ok(function.finish(vec![slot_offset], &mut module.funcs))
}

/// Compute where an enum's storage ends as a tuple (tail_slot_ptr, tail_slot_offset)
///
/// Arguments:
/// - `head_slot_ptr` (i32): pointer to the start slot (U256 big-endian)
/// - `head_slot_offset` (i32): byte offset (0-31) where the enum starts in the slot
///
/// Returns:
/// - `tail_slot_ptr` (i32): pointer to the end slot (U256 big-endian)
/// - `tail_slot_offset` (i32): byte offset (0-31) where the enum ends
///
/// Behavior:
/// - If enum fits in current slot: (head_slot_ptr, head_slot_offset + enum_size)
/// - If not: advances slots as needed so offset wraps to final position.
pub fn compute_enum_storage_tail_position(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::ComputeEnumStorageTailPosition
        .get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let head_slot_ptr = module.locals.add(ValType::I32);
    let head_slot_offset = module.locals.add(ValType::I32);

    // Get the storage size function for this enum type
    let get_storage_size_by_offset_fn =
        RuntimeFunction::GetStorageSizeByOffset.get_generic(module, compilation_ctx, &[itype])?;

    // Compute enum_size using the offset
    let enum_size = module.locals.add(ValType::I32);
    builder
        .local_get(head_slot_offset)
        .call(get_storage_size_by_offset_fn)
        .local_set(enum_size);

    let tail_slot_offset = module.locals.add(ValType::I32);
    let tail_slot_ptr = module.locals.add(ValType::I32);

    // Allocate 36 bytes, 32 for the slot and 4 for the offset
    builder
        .i32_const(36)
        .call(compilation_ctx.allocator)
        .local_tee(tail_slot_ptr);

    // Copy the head slot to the first 32 bytes of the data pointer
    builder
        .local_get(head_slot_ptr)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // free_bytes = 32 - head_slot_offset
    let free_bytes = module.locals.add(ValType::I32);
    builder
        .i32_const(SLOT_SIZE as i32)
        .local_get(head_slot_offset)
        .binop(BinaryOp::I32Sub)
        .local_set(free_bytes);

    let mut inner_result = Ok(());
    builder
        .local_get(enum_size)
        .local_get(free_bytes)
        .binop(BinaryOp::I32GeU)
        .if_else(
            None,
            |then| {
                // Case: enum_size >= free_bytes, so it will span multiple slots
                // 1) *tail_slot_ptr = *head_slot_ptr + ((enum_size - free_bytes) / 32) as u256 LE

                // delta_slot_ptr = (enum_size - free_bytes) / 32 as u256 LE (how many slots to add to the current slot)
                let delta_slot_ptr = module.locals.add(ValType::I32);
                // Allocate 32 bytes for the slot offset
                then.i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(delta_slot_ptr);

                // (enum_size - free_bytes) / 32 as u32
                then.local_get(enum_size)
                    .local_get(free_bytes)
                    .binop(BinaryOp::I32Sub)
                    .i32_const(SLOT_SIZE as i32)
                    .binop(BinaryOp::I32DivS)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add);

                // Store the offset in the first 4 bytes to make it a u256 LE
                then.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                inner_result = (|| {
                    // Swap the end slot from BE to LE for addition
                    let swap_256_fn =
                        RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx))?;
                    then.local_get(tail_slot_ptr)
                        .local_get(tail_slot_ptr)
                        .call(swap_256_fn);

                    // Add the offset to the end slot (right now equal to the current slot)
                    let add_u256_fn =
                        RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx))?;

                    then.local_get(delta_slot_ptr)
                        .local_get(tail_slot_ptr)
                        .local_get(tail_slot_ptr)
                        .i32_const(32)
                        .call(add_u256_fn)
                        .local_set(tail_slot_ptr);

                    // Swap back to BE
                    then.local_get(tail_slot_ptr)
                        .local_get(tail_slot_ptr)
                        .call(swap_256_fn);

                    // 2) tail_slot_offset = (enum_size - free_bytes) % 32
                    then.local_get(enum_size)
                        .local_get(free_bytes)
                        .binop(BinaryOp::I32Sub)
                        .i32_const(SLOT_SIZE as i32)
                        .binop(BinaryOp::I32RemS)
                        .local_set(tail_slot_offset);

                    Ok(())
                })();
            },
            |else_| {
                // Case: enum_size < free_bytes, so it fits entirely in the current slot
                // 1) end_slot = start_slot (already set by the copy above)
                // 2) tail_slot_offset = head_slot_offset + enum_size
                else_
                    .local_get(head_slot_offset)
                    .local_get(enum_size)
                    .binop(BinaryOp::I32Add)
                    .local_set(tail_slot_offset);
            },
        );

    inner_result?;

    // Store the tail offset in the last 4 bytes of the data pointer
    builder
        .local_get(tail_slot_ptr)
        .local_get(tail_slot_offset)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 32,
            },
        );

    // Return the pointer
    builder.local_get(tail_slot_ptr);

    Ok(function.finish(vec![head_slot_ptr, head_slot_offset], &mut module.funcs))
}

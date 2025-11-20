//! This module implements the logic to encode/decode data in storage slots.
//!
//! The encoding used is the same as the one used by Solidity.
//! For more information:
//! https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET, DATA_ZERO_OFFSET},
    hostio::host_functions::{native_keccak256, storage_cache_bytes32, storage_load_bytes32},
    native_functions::object::add_delete_field_instructions,
    runtime::RuntimeFunction,
    storage::storage_layout::field_size,
    translation::intermediate_types::IntermediateType,
    vm_handled_types::is_uid_or_named_id,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::error::{EncodeError, StorageError};

/// Emits WASM instructions that encode a struct and write it into storage.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `struct_ptr`: Local holding the heap pointer to the struct in memory.
/// - `slot_ptr`: Local pointing to the base storage slot where the struct is saved.
/// - `slot_offset`: Local used as a running counter of bytes already written in the current slot.
/// - `owner_ptr`: Optional local to the owner struct UID. If the struct has `key`, this will be `None`.
/// - `itype`: Intermediate type of the struct to encode.
///
/// Returns:
/// - None. Writes encoded bytes to storage via the builder.
#[allow(clippy::too_many_arguments)]
pub fn add_encode_and_save_into_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    struct_ptr: LocalId,
    slot_ptr: LocalId,
    slot_offset: LocalId,
    owner_ptr: Option<LocalId>,
    itype: &IntermediateType,
) -> Result<(), StorageError> {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);

    // Runtime functions
    let get_struct_id_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx))?;
    let accumulate_or_advance_slot_write_fn =
        RuntimeFunction::AccumulateOrAdvanceSlotWrite.get(module, Some(compilation_ctx))?;

    // Get the IStruct representation
    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    let field_owner_ptr = module.locals.add(ValType::I32);

    if struct_.has_key {
        // Set the slot data to zero
        builder
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(DATA_ZERO_OFFSET)
            .i32_const(32)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

        // Save the type hash in the slot data at offset 24 (last 8 bytes)
        builder
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i64_const(itype.get_hash(compilation_ctx) as i64)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 24,
                },
            );

        // Update the slot_offset to reflect the 8-byte type hash
        builder.i32_const(8).local_set(slot_offset);

        // If the current struct has the key ability, its struct id is used as the owner for wrapped objects.
        // Otherwise, use the owner passed as argument.
        builder
            .local_get(struct_ptr)
            .call(get_struct_id_fn)
            .local_set(field_owner_ptr);
    } else if let Some(owner_ptr) = owner_ptr {
        builder.local_get(owner_ptr).local_set(field_owner_ptr);
    } else {
        builder.unreachable();
    }

    for (index, field) in struct_.fields.iter().enumerate() {
        if is_uid_or_named_id(&field, compilation_ctx)? {
            // UIDs are not written in storage, except for referencing nested child structs (wrapped objects).
            continue;
        }
        let field_size = field_size(field, compilation_ctx)? as i32;
        // Update the slot_offset to include the field size.
        // If we've filled the current slot, cache its data and move to the next slot.
        builder
            .local_get(slot_ptr)
            .local_get(slot_offset)
            .i32_const(field_size)
            .call(accumulate_or_advance_slot_write_fn)
            .local_set(slot_offset);

        // Load field's intermediate pointer
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        // Encode the field
        add_encode_intermediate_type_instructions(
            module,
            builder,
            compilation_ctx,
            slot_ptr,
            slot_offset,
            field_owner_ptr,
            field,
            true,
        )?;
    }

    // Always save the last slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);

    Ok(())
}

/// Emits WASM instructions that encode a tagged enum and write the active
/// variant into storage.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `enum_ptr`: Local holding the heap pointer to the enum in memory.
/// - `slot_ptr`: Local pointing to the base storage slot where the enum is saved.
/// - `slot_offset`: Local used as a running counter of bytes already written in the current slot.
/// - `owner_ptr`: Local to the owner struct UID (for nested keyed objects).
/// - `itype`: Intermediate type of the enum to encode.
///
/// Returns:
/// - None. Writes encoded bytes to storage via the builder.
#[allow(clippy::too_many_arguments)]
pub fn add_encode_and_save_into_storage_enum_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    enum_ptr: LocalId,
    slot_ptr: LocalId,
    slot_offset: LocalId,
    owner_ptr: LocalId,
    itype: &IntermediateType,
) -> Result<(), StorageError> {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);

    // Runtime functions
    let accumulate_or_advance_slot_write_fn =
        RuntimeFunction::AccumulateOrAdvanceSlotWrite.get(module, Some(compilation_ctx))?;
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx))?;
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx))?;
    let compute_enum_storage_tail_position_fn = RuntimeFunction::ComputeEnumStorageTailPosition
        .get_generic(module, compilation_ctx, &[itype])?;

    // Get the IEnum representation
    let enum_ = compilation_ctx.get_enum_by_intermediate_type(itype)?;

    // Compute the tail slot and tail offset for the enum

    let tail_slot_ptr = module.locals.add(ValType::I32);

    builder
        .local_get(slot_ptr)
        .local_get(slot_offset)
        .call(compute_enum_storage_tail_position_fn)
        .local_set(tail_slot_ptr);

    let variant_index = module.locals.add(ValType::I32);
    builder
        .local_get(enum_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(variant_index);

    // Match on the variant and encode its fields.
    let mut inner_result = Ok(());
    enum_.match_on_variant(builder, variant_index, |variant, block| {
        // Update the slot_offset to account for the variant index size.
        block
            .local_get(slot_ptr)
            .local_get(slot_offset)
            .i32_const(1)
            .call(accumulate_or_advance_slot_write_fn)
            .local_set(slot_offset);

        // Write the variant index in the slot data.
        block
            .add_slot_data_ptr_plus_offset(slot_offset)
            .local_get(variant_index)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32_8 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        for (index, field) in variant.fields.iter().enumerate() {
            let field_size = field_size(field, compilation_ctx);

            let field_size = match field_size {
                Ok(fs) => fs as i32,
                Err(e) => {
                    inner_result = Err(e);
                    break;
                }
            };
            // Update the slot_offset to include the field size.
            // If we've filled the current slot, cache its data and move to the next slot.
            block
                .local_get(slot_ptr)
                .local_get(slot_offset)
                .i32_const(field_size)
                .call(accumulate_or_advance_slot_write_fn)
                .local_set(slot_offset);

            // Load field's intermediate pointer
            block.local_get(enum_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4 + 4 * index as u32,
                },
            );

            // Encode the field
            inner_result = add_encode_intermediate_type_instructions(
                module,
                block,
                compilation_ctx,
                slot_ptr,
                slot_offset,
                owner_ptr,
                field,
                true,
            );

            if inner_result.is_err() {
                break;
            }
        }

        // Handle leftover storage from large enum variants.
        // Each enum reserves space for its largest variant, so writing a smaller variant
        // may leave extra slots containing old data. We need to clear these slots
        // to avoid storing stale bytes.
        // If we're not at the tail slot (slot_ptr != tail_slot_ptr):
        //   - Save the current slot data, wipe the slot data and advance to the next slot.
        //   - If the next slot is not the tail slot, wipe it and advance to the next slot. Repeat.
        block
            .local_get(slot_ptr)
            .local_get(tail_slot_ptr)
            .i32_const(32)
            .call(equality_fn)
            .if_else(
                None,
                |_| {
                    // slot_ptr == tail_slot_ptr, do nothing
                    // the slot data is going to be cached by the caller of this function
                },
                |else_| {
                    // slot_ptr != tail_slot_ptr - first iteration
                    // Cache the current slot's data at DATA_SLOT_DATA_PTR_OFFSET
                    else_
                        .local_get(slot_ptr)
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .call(storage_cache);

                    // Wipe DATA_SLOT_DATA_PTR_OFFSET (set to zero)
                    else_
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .i32_const(DATA_ZERO_OFFSET)
                        .i32_const(32)
                        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                    // Advance to the next slot
                    else_
                        .local_get(slot_ptr)
                        .call(next_slot_fn)
                        .local_set(slot_ptr);

                    // Cache the now-empty (wiped) DATA_SLOT_DATA_PTR_OFFSET at the new slot_ptr
                    else_
                        .local_get(slot_ptr)
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .call(storage_cache);

                    // Loop: continue advancing and caching zero data until slot_ptr == tail_slot_ptr
                    else_.loop_(None, |loop_| {
                        let loop_id = loop_.id();

                        // Check if slot_ptr == tail_slot_ptr
                        loop_
                            .local_get(slot_ptr)
                            .local_get(tail_slot_ptr)
                            .i32_const(32)
                            .call(equality_fn)
                            .if_else(
                                None,
                                |_| {
                                    // slot_ptr == tail_slot_ptr, exit the loop
                                },
                                |inner_else| {
                                    // slot_ptr != tail_slot_ptr
                                    // Advance to the next slot
                                    inner_else
                                        .local_get(slot_ptr)
                                        .call(next_slot_fn)
                                        .local_set(slot_ptr);

                                    // Cache DATA_ZERO_OFFSET directly to the new slot_ptr
                                    inner_else
                                        .local_get(slot_ptr)
                                        .i32_const(DATA_ZERO_OFFSET)
                                        .call(storage_cache);

                                    // Continue the loop
                                    inner_else.br(loop_id);
                                },
                            );
                    });
                },
            );
    });

    inner_result?;

    // slot_offset = tail_slot_offset
    builder
        .local_get(tail_slot_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 32,
            },
        )
        .local_set(slot_offset);

    Ok(())
}

/// Emits WASM instructions to encode a vector and write it into storage,
/// including truncation of any stale elements that existed previously in storage.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `vector_ptr`: Local holding the heap pointer to the vector in memory.
/// - `slot_ptr`: Local pointing to the vector header slot in storage.
/// - `owner_ptr`: Local to the owner struct UID (for nested keyed objects).
/// - `inner`: Intermediate type of the vector elements.
///
/// Returns:
/// - None. Writes encoded bytes to storage via the builder.
pub fn add_encode_and_save_into_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    vector_ptr: LocalId,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    inner: &IntermediateType,
) -> Result<(), StorageError> {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (storage_load, _) = storage_load_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None)?;
    let derive_dyn_array_slot_fn =
        RuntimeFunction::DeriveDynArraySlot.get(module, Some(compilation_ctx))?;
    let accumulate_or_advance_slot_write_fn =
        RuntimeFunction::AccumulateOrAdvanceSlotWrite.get(module, Some(compilation_ctx))?;

    // Locals
    let elem_slot_ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    // Stack size of the inner type
    let stack_size = inner
        .stack_data_size()
        .map_err(|e| EncodeError::IntermediateType(e.into()))? as i32;

    // Element size in storage
    let elem_size = field_size(inner, compilation_ctx)? as i32;

    // Wipe the data so we write on it safely
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(0)
        .i32_const(32)
        .memory_fill(compilation_ctx.memory_id);

    // Allocate 32 bytes for the element slot
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(elem_slot_ptr);

    // Load vector length from its header
    builder
        .local_get(vector_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // When elements are popped from the vector, they must be cleared from storage.
    // The encoding process updates the storage to reflect the current vector state (in-memory), but if the original vector (currently stored) was longer,
    // any leftover data will persist unless explicitly removed.
    let old_len = module.locals.add(ValType::I32);

    // Load original vector header slot data
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    // Get the original length of the vector
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        )
        .call(swap_fn)
        .local_set(old_len);

    // Set aux locals for looping
    let i = module.locals.add(ValType::I32);
    let elem_slot_offset = module.locals.add(ValType::I32);

    let mut inner_result = Ok(());
    // If the old length is greater than the new length, we need to delete those residual elements from the storage.
    builder.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if the old length is greater than the new length else skip the rest of the instructions.
        outer_block
            .local_get(old_len)
            .local_get(len)
            .binop(BinaryOp::I32LeU)
            .br_if(outer_block_id);

        outer_block.block(None, |inner_block| {
            let inner_block_id = inner_block.id();

            // Set the index to the current length of the vector,
            // to start deleting from there on.
            inner_block.local_get(len).local_set(i);

            let elem_size_local = module.locals.add(ValType::I32);
            inner_block.i32_const(elem_size).local_set(elem_size_local);

            // Compute the slot for the first element after the current vector's last element
            inner_block
                .local_get(slot_ptr)
                .local_get(len)
                .local_get(elem_size_local)
                .local_get(elem_slot_ptr)
                .call(derive_dyn_array_slot_fn);

            inner_block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // Delete the field from the storage
                inner_result = add_delete_field_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_slot_ptr,
                    elem_slot_offset,
                    inner,
                    elem_size,
                );

                // Exit after processing all elements (from len to old_len)
                loop_
                    .local_get(i)
                    .local_get(old_len)
                    .i32_const(1)
                    .binop(BinaryOp::I32Sub)
                    .binop(BinaryOp::I32GeU)
                    .br_if(inner_block_id);

                // i = i + 1 and continue the loop
                loop_
                    .local_get(i)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_set(i)
                    .br(loop_id);
            });
        });

        // Wipe out the last slot before exiting
        // This ensures that the last slot is always deleted, as the add_delete_field_instructions
        // only deletes the slot if all the bytes in the slot are used.
        outer_block
            .local_get(elem_slot_ptr)
            .i32_const(DATA_ZERO_OFFSET)
            .call(storage_cache);
    });

    inner_result?;

    // Reset the aux locals for the next loop
    // This loop encodes and saves the vector in memory to the storage.
    builder.i32_const(0).local_set(i);
    builder.i32_const(0).local_set(elem_slot_offset);

    let mut inner_result = Ok(());
    // Loop through the vector and encode and save the elements to the storage.
    builder.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if length == 0
        outer_block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(outer_block_id);

        // First slot = keccak(header_slot)
        outer_block
            .local_get(slot_ptr)
            .i32_const(32)
            .local_get(elem_slot_ptr)
            .call(native_keccak);

        outer_block.block(None, |inner_block| {
            let inner_block_id = inner_block.id();

            inner_block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // Update the slot_offset to include the field size.
                // If we've filled the current slot, cache its data and move to the next slot.
                loop_
                    .local_get(elem_slot_ptr)
                    .local_get(elem_slot_offset)
                    .i32_const(elem_size)
                    .call(accumulate_or_advance_slot_write_fn)
                    .local_set(elem_slot_offset);

                // Pointer to the element in memory
                loop_.vec_elem_ptr(vector_ptr, i, stack_size).load(
                    compilation_ctx.memory_id,
                    if stack_size == 8 {
                        LoadKind::I64 { atomic: false }
                    } else {
                        LoadKind::I32 { atomic: false }
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // Encode the intermediate type
                inner_result = add_encode_intermediate_type_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_slot_ptr,
                    elem_slot_offset,
                    owner_ptr,
                    inner,
                    false,
                );

                // Exit after processing all elements
                loop_
                    .local_get(i)
                    .local_get(len)
                    .i32_const(1)
                    .binop(BinaryOp::I32Sub)
                    .binop(BinaryOp::I32Eq)
                    .br_if(inner_block_id);

                // i = i + 1 and continue the loop
                loop_
                    .local_get(i)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_set(i)
                    .br(loop_id);
            });
        });

        // Store the last element.
        // If the element is a vector, here we will be storing the length of it in it's header slot
        outer_block
            .local_get(elem_slot_ptr)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_cache);
    });

    inner_result?;

    // Wipe the data so we can write the length of the vector
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(0)
        .i32_const(32)
        .memory_fill(compilation_ctx.memory_id);

    // Write the length in the slot data. This will be cached to the storage by the caller.
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .local_get(len)
        .call(swap_fn)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        );

    Ok(())
}

/// Emits WASM instructions to encode an intermediate type and write it into a
/// storage slot.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `slot_ptr`: Local pointing to the base storage slot to write to.
/// - `slot_offset`: Local used as a running counter of bytes already written in the current slot.
/// - `owner_ptr`: Local to the owner struct UID (relevant for keyed objects).
/// - `itype`: Intermediate type to encode.
/// - `is_field`: Whether the source value is a struct/enum field (affects extra loads for stack types).
///
/// Stack expectations:
/// - The pointer/value to encode must be on the stack when this function is called.
#[allow(clippy::too_many_arguments)]
pub fn add_encode_intermediate_type_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    slot_offset: LocalId,
    owner_ptr: LocalId,
    itype: &IntermediateType,
    is_field: bool,
) -> Result<(), StorageError> {
    // Locals
    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);

    // Stack and storage size of the type
    let stack_size = itype
        .stack_data_size()
        .map_err(|e| EncodeError::IntermediateType(e.into()))? as i32;
    let storage_size = field_size(itype, compilation_ctx)? as i32;

    // Runtime functions
    let get_struct_id_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx))?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx))?;

    match itype {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64 => {
            let (val, load_kind, swap_fn) = if stack_size == 8 {
                let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None)?;
                (val_64, LoadKind::I64 { atomic: false }, swap_fn)
            } else {
                let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None)?;
                (val_32, LoadKind::I32 { atomic: false }, swap_fn)
            };

            // If we are processing a field from a struct, a second load is needed.
            // This is because structs store pointers to their fields, even for non-heap types.
            if is_field {
                builder.load(
                    compilation_ctx.memory_id,
                    load_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }

            // Convert the value to big endian
            builder.call(swap_fn);

            // We need to shift the swapped bytes to the right because WASM is little endian. If we try
            // to write a 16 bits number contained in a 32 bits number, without shifting, it will write
            // the zeroed part.
            // This only needs to be done for 32 bits (4 bytes) numbers
            if stack_size == 4 {
                if storage_size == 1 {
                    builder.i32_const(24).binop(BinaryOp::I32ShrU);
                } else if storage_size == 2 {
                    builder.i32_const(16).binop(BinaryOp::I32ShrU);
                }
            }

            builder.local_set(val);

            let store_kind = if storage_size == 1 {
                StoreKind::I32_8 { atomic: false }
            } else if storage_size == 2 {
                StoreKind::I32_16 { atomic: false }
            } else if storage_size == 4 {
                StoreKind::I32 { atomic: false }
            } else {
                StoreKind::I64 { atomic: false }
            };

            // Save the value in slot data
            builder
                .add_slot_data_ptr_plus_offset(slot_offset)
                .local_get(val)
                .store(
                    compilation_ctx.memory_id,
                    store_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
        }
        IntermediateType::IU128 => {
            let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx))?;

            // Slot data plus offset as dest ptr
            builder.add_slot_data_ptr_plus_offset(slot_offset);

            // Transform to BE
            builder.call(swap_fn);
        }
        IntermediateType::IU256 => {
            let swap_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx))?;

            // Slot data plus offset as dest ptr (offset should be zero because data is already
            // 32 bytes in size)
            builder.add_slot_data_ptr_plus_offset(slot_offset);

            // Transform to BE
            builder.call(swap_fn);
        }
        IntermediateType::IAddress | IntermediateType::ISigner => {
            // We need to swap values before copying because memory copy takes dest pointer
            // first
            builder.local_set(val_32);

            // Slot data plus offset as dest ptr
            builder.add_slot_data_ptr_plus_offset(slot_offset);

            // Grab the last 20 bytes of the address
            builder
                .local_get(val_32)
                .i32_const(12)
                .binop(BinaryOp::I32Add);

            // Amount of bytes to copy
            builder.i32_const(20);

            builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
            // This section handles encoding of nested structs within parent structs.
            // The behavior differs based on whether the child struct has the 'key' ability:
            // - If child has 'key': stored as separate object under the parent key
            // - If child has no 'key': flattened into parent struct's data

            let child_struct_ptr = module.locals.add(ValType::I32);
            builder.local_set(child_struct_ptr);

            // Get child struct by (module_id, index)
            let child_struct = compilation_ctx.get_struct_by_intermediate_type(itype)?;

            if child_struct.has_key {
                // ====================================================================
                // CHILD STRUCT WITH KEY - Store as Separate Object
                // ====================================================================
                // When a child struct has the 'key' ability, it becomes a separate
                // object in storage rather than being flattened into the parent.
                // This requires:
                // 1. Calculating the slot for the child struct
                // 2. Recursively encoding the child struct in its own slot
                // 3. Storing the child struct UID in the parent's data

                // Get the child struct UID
                let child_struct_id_ptr = module.locals.add(ValType::I32);
                builder
                    .local_get(child_struct_ptr)
                    .call(get_struct_id_fn)
                    .local_set(child_struct_id_ptr);

                // Calculate the child struct slot
                builder
                    .local_get(owner_ptr)
                    .local_get(child_struct_id_ptr)
                    .call(write_object_slot_fn);

                // Allocate memory for the child struct slot and copy the calculated
                // slot data to avoid overwriting during recursive encoding.
                let child_struct_slot_ptr = module.locals.add(ValType::I32);
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(child_struct_slot_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Reset slot_offset for the child struct encoding
                builder.i32_const(0).local_set(slot_offset);

                // Recursively encode and store the child struct
                add_encode_and_save_into_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    child_struct_ptr,
                    child_struct_slot_ptr,
                    slot_offset,
                    None,
                    itype,
                )?;

                // After encoding the child struct, we need to store its UID in the
                // parent struct's data so the parent can reference the child.
                // The UID takes exactly 32 bytes (one full slot).

                // Update slot_offset to reflect the 32-byte UID
                builder.i32_const(32).local_set(slot_offset);

                // Copy the child struct UID to the parent's data section
                // This creates the reference from parent to child struct
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .local_get(child_struct_id_ptr)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            } else {
                // ====================================================================
                // CHILD STRUCT WITHOUT KEY - Flatten into Parent
                // ====================================================================
                // When a child struct doesn't have the 'key' ability, it gets
                // flattened directly into the parent struct's data. This means
                // all fields of the child struct are stored inline within the
                // parent struct's storage slot.

                add_encode_and_save_into_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    child_struct_ptr,
                    slot_ptr,
                    slot_offset,
                    Some(owner_ptr),
                    itype,
                )?;
            }
        }
        IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
            builder.local_set(val_32);

            add_encode_and_save_into_storage_enum_instructions(
                module,
                builder,
                compilation_ctx,
                val_32,
                slot_ptr,
                slot_offset,
                owner_ptr,
                itype,
            )?;
        }
        IntermediateType::IVector(inner) => {
            builder.local_set(val_32);

            add_encode_and_save_into_storage_vector_instructions(
                module,
                builder,
                compilation_ctx,
                val_32,
                slot_ptr,
                owner_ptr,
                inner,
            )?;

            builder.i32_const(32).local_set(slot_offset);
        }
        _ => return Err(EncodeError::InvalidType(itype.clone()))?,
    };

    Ok(())
}

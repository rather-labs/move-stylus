//! This module implements the logic to encode/decode data in storage slots.
//!
//! The encoding used is the same as the one used by Solidity.
//! For more information:
//! https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET, DATA_ZERO_OFFSET},
    hostio::host_functions::{native_keccak256, storage_cache_bytes32, storage_load_bytes32},
    native_functions::object::add_delete_field_instructions,
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, vector::IVector},
    wasm_builder_extensions::WasmBuilderExtension,
};
/// Adds the instructions to encode and save into storage an specific struct.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `struct_ptr` - pointer to the struct to be encoded
/// `slot_ptr` - storage's slot where the data will be saved
/// `owner_ptr` - Optional pointer to the owner struct id. If the struct has key this will be None.
/// `itype` - intermediate type of the struct to be encoded and saved
/// `written_bytes_in_slot` - number of bytes already written in the slot. This will be != 0 if
/// this function is recusively called to save a struct inside another struct.
#[allow(clippy::too_many_arguments)]
pub fn add_encode_and_save_into_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    struct_ptr: LocalId,
    slot_ptr: LocalId,
    owner_ptr: Option<LocalId>,
    itype: &IntermediateType,
    written_bytes_in_slot: LocalId,
) {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);

    // Runtime functions
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));
    let get_struct_id_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));

    // Get the IStruct representation
    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap_or_else(|_| {
            panic!("there wsas an error encoding an struct for storage, found {itype:?}")
        });

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

        // Update written bytes counter to reflect the 8-byte type hash
        builder.i32_const(8).local_set(written_bytes_in_slot);

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
        if field.is_uid_or_named_id(compilation_ctx) {
            // UIDs are not written in storage, except for referencing nested child structs (wrapped objects).
            continue;
        }
        let field_size = field_size(field, compilation_ctx);
        builder
            .local_get(written_bytes_in_slot)
            .i32_const(field_size as i32)
            .binop(BinaryOp::I32Add)
            .i32_const(32)
            .binop(BinaryOp::I32GtS)
            .if_else(
                None,
                |then| {
                    // Save previous slot
                    then.local_get(slot_ptr)
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .call(storage_cache);

                    // Wipe the data so it can be filled with new data
                    then.i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .i32_const(0)
                        .i32_const(32)
                        .memory_fill(compilation_ctx.memory_id);

                    // Next slot
                    then.local_get(slot_ptr)
                        .call(next_slot_fn)
                        .local_set(slot_ptr);

                    // Set the written bytes in slot to the field size
                    then.i32_const(field_size as i32)
                        .local_set(written_bytes_in_slot);
                },
                |else_| {
                    // Increment the written bytes in slot by the field size
                    else_
                        .local_get(written_bytes_in_slot)
                        .i32_const(field_size as i32)
                        .binop(BinaryOp::I32Add)
                        .local_set(written_bytes_in_slot);
                },
            );

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
            field_owner_ptr,
            field,
            written_bytes_in_slot,
            true,
        );
    }

    // Always save the last slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);
}

#[allow(clippy::too_many_arguments)]
pub fn add_encode_and_save_into_storage_enum_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    enum_ptr: LocalId,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    itype: &IntermediateType,
    written_bytes_in_slot: LocalId,
) {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);

    // Runtime functions
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Get the IEnum representation
    let enum_ = compilation_ctx
        .get_enum_by_intermediate_type(itype)
        .unwrap_or_else(|_| {
            panic!("there wsas an error encoding an enum for storage, found {itype:?}")
        });

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

    // First write the variant index in the slot data.
    // written_bytes_in_slot already accounts for the variant's 1 byte.
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(32)
        .local_get(written_bytes_in_slot)
        .binop(BinaryOp::I32Sub)
        .binop(BinaryOp::I32Add)
        .local_get(variant_index)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Match on the variant and encode its fields.
    enum_.match_on_variant(builder, variant_index, |variant, block| {
        for (index, field) in variant.fields.iter().enumerate() {
            let field_size = field_size(field, compilation_ctx);
            block
                .local_get(written_bytes_in_slot)
                .i32_const(field_size as i32)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32GtS)
                .if_else(
                    None,
                    |then| {
                        // Save previous slot
                        then.local_get(slot_ptr)
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .call(storage_cache);

                        // Wipe the data so it can be filled with new data
                        then.i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .i32_const(0)
                            .i32_const(32)
                            .memory_fill(compilation_ctx.memory_id);

                        // Next slot
                        then.local_get(slot_ptr)
                            .call(next_slot_fn)
                            .local_set(slot_ptr);

                        // Set the written bytes in slot to the field size
                        then.i32_const(field_size as i32)
                            .local_set(written_bytes_in_slot);
                    },
                    |else_| {
                        // Increment the written bytes in slot by the field size
                        else_
                            .local_get(written_bytes_in_slot)
                            .i32_const(field_size as i32)
                            .binop(BinaryOp::I32Add)
                            .local_set(written_bytes_in_slot);
                    },
                );

            // Load field's intermediate pointer
            block.local_get(enum_ptr).load(
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
                block,
                compilation_ctx,
                slot_ptr,
                owner_ptr,
                field,
                written_bytes_in_slot,
                true,
            );
        }

        // Always save the last slot
        block
            .local_get(slot_ptr)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_cache);
    });
}

/// Adds the instructions to read, decode from storage and build in memory a structure.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `slot_ptr` - storage's slot where the data will be saved
/// `struct_id_ptr` - optional pointer to the struct id. If the struct does not have the key ability, this will be None.
/// `owner_ptr` - pointer to the owner struct id.
/// `struct_` - structural information of the struct to be encoded and saved
/// `read_bytes_in_slot` - number of bytes already read in the slot.
/// another struct.
///
/// # Returns
/// pointer where the read struct is allocated
#[allow(clippy::too_many_arguments)]
pub fn add_read_and_decode_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    struct_id_ptr: Option<LocalId>,
    owner_ptr: LocalId,
    itype: &IntermediateType,
    read_bytes_in_slot: LocalId,
) -> LocalId {
    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);

    // Runtime functions
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Get the IStruct representation
    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap_or_else(|_| {
            panic!("there wsas an error encoding an struct for storage, found {itype:?}")
        });

    // Locals
    let struct_ptr = module.locals.add(ValType::I32);
    let field_ptr = module.locals.add(ValType::I32);
    let field_owner_ptr = module.locals.add(ValType::I32);

    // If the struct has the key ability
    if struct_.has_key {
        // Prepend the owner to the struct memory representation
        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_get(owner_ptr)
            .i32_const(32)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

        // Check if the type hash is the same as the one in the storage

        // i. Retrieve the initial slot from storage, which contains the type hash in the first 8 bytes
        builder
            .local_get(slot_ptr)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 24,
                },
            );

        // ii. Hash the type and compare it with the retrieved one from the storage
        builder
            .i64_const(itype.get_hash(compilation_ctx) as i64)
            .binop(BinaryOp::I64Eq)
            .if_else(
                None,
                |then| {
                    // If they match, set the read bytes in slot to 8
                    then.i32_const(8).local_set(read_bytes_in_slot);
                },
                |else_| {
                    // If they don't match, trap
                    else_.unreachable();
                },
            );

        // Set the owner for the fields (wrapped objects).
        // If the struct has key, then the owner is the struct id.
        // Otherwise, use the owner pointer passed as argument.
        if let Some(struct_id_ptr) = struct_id_ptr {
            builder.local_get(struct_id_ptr).local_set(field_owner_ptr);
        } else {
            builder.unreachable();
        }
    } else {
        builder.local_get(owner_ptr).local_set(field_owner_ptr);
    }

    // Allocate memory for the struct
    // For structs with key ability, the owner is already prepended in memory
    builder
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(struct_ptr);

    // Iterate through the fields of the struct
    for (index, field) in struct_.fields.iter().enumerate() {
        // If the field is a UID or NamedId, don't call decode_intermediate_type_instructions and process it here
        if field.is_uid_or_named_id(compilation_ctx) {
            // Save the struct pointer before the UID
            builder
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_get(struct_ptr)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Wrap the UID bytes and store the wrapper at the field pointer
            // This mimics the UID struct representation in memory

            let struct_id_ptr_wrapper = module.locals.add(ValType::I32);

            // Allocate 4 bytes for the field pointer
            builder
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_set(field_ptr);

            // Allocate 4 bytes for the UID wrapper
            builder
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_set(struct_id_ptr_wrapper);

            if let Some(struct_id_ptr) = struct_id_ptr {
                // Store the UID at the UID wrapper pointer
                builder
                    .local_get(struct_id_ptr_wrapper)
                    .local_get(struct_id_ptr)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            };

            // Store the UID wrapper at the field pointer
            builder
                .local_get(field_ptr)
                .local_get(struct_id_ptr_wrapper)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
        } else {
            let field_size = field_size(field, compilation_ctx) as i32;
            // If the entire slot has been processed, move to the subsequent slot and retrieve its data
            builder
                .local_get(read_bytes_in_slot)
                .i32_const(field_size)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32GtU)
                .if_else(
                    None,
                    |then| {
                        then.local_get(slot_ptr)
                            .call(next_slot_fn)
                            .local_set(slot_ptr);

                        // Load the slot data
                        then.local_get(slot_ptr)
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .call(storage_load);

                        then.i32_const(field_size).local_set(read_bytes_in_slot);
                    },
                    |else_| {
                        else_
                            .local_get(read_bytes_in_slot)
                            .i32_const(field_size)
                            .binop(BinaryOp::I32Add)
                            .local_set(read_bytes_in_slot);
                    },
                );

            // Decode the field according to its type
            add_decode_intermediate_type_instructions(
                module,
                builder,
                compilation_ctx,
                field_ptr,
                slot_ptr,
                field_owner_ptr,
                field,
                read_bytes_in_slot,
            );
        }
        // Store the field in the struct at offset index * 4
        builder.local_get(struct_ptr).local_get(field_ptr).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );
    }

    struct_ptr
}

#[allow(clippy::too_many_arguments)]
pub fn add_read_and_decode_storage_enum_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    itype: &IntermediateType,
    read_bytes_in_slot: LocalId,
) -> LocalId {
    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);

    // Runtime functions
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Get the IEnum representation
    let enum_ = compilation_ctx
        .get_enum_by_intermediate_type(itype)
        .expect("enum not found");

    let heap_size = enum_
        .heap_size
        .expect("cannot decode enum with unresolved generic heap size") as i32;

    // Locals
    let enum_ptr = module.locals.add(ValType::I32);
    let field_ptr = module.locals.add(ValType::I32);
    let variant_index = module.locals.add(ValType::I32);

    // Allocate memory for the enum
    builder
        .i32_const(heap_size)
        .call(compilation_ctx.allocator)
        .local_set(enum_ptr);

    // Read the variant index (first byte)
    // The read_bytes_in_slot variable already accounts for the variant index byte.
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(32)
        .local_get(read_bytes_in_slot)
        .binop(BinaryOp::I32Sub)
        .binop(BinaryOp::I32Add)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(variant_index);

    // Store the variant index in the first 4 bytes of the enum
    builder.local_get(enum_ptr).local_get(variant_index).store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Decode fields for the active variant
    enum_.match_on_variant(builder, variant_index, |variant, block| {
        for (index, field) in variant.fields.iter().enumerate() {
            let field_size = field_size(field, compilation_ctx) as i32;

            // If the entire slot has been processed, move to the subsequent slot and retrieve its data
            block
                .local_get(read_bytes_in_slot)
                .i32_const(field_size)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32GtU)
                .if_else(
                    None,
                    |then| {
                        then.local_get(slot_ptr)
                            .call(next_slot_fn)
                            .local_set(slot_ptr);

                        // Load the slot data
                        then.local_get(slot_ptr)
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .call(storage_load);

                        then.i32_const(field_size).local_set(read_bytes_in_slot);
                    },
                    |else_| {
                        else_
                            .local_get(read_bytes_in_slot)
                            .i32_const(field_size)
                            .binop(BinaryOp::I32Add)
                            .local_set(read_bytes_in_slot);
                    },
                );

            // Decode the field according to its type
            add_decode_intermediate_type_instructions(
                module,
                block,
                compilation_ctx,
                field_ptr,
                slot_ptr,
                owner_ptr,
                field,
                read_bytes_in_slot,
            );

            // Store the field in the enum at offset index * 4 (after the 4-byte tag)
            block.local_get(enum_ptr).local_get(field_ptr).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4 + 4 * index as u32,
                },
            );
        }
    });

    enum_ptr
}

/// Adds the instructions to encode and save a vector (as a field in a struct) into storage.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `vector_ptr` - pointer to the vector in memory
/// `slot_ptr` - pointer to the vector header slot
/// `owner_ptr` - pointer to the owner struct id.
/// `inner` - inner type of the vector
pub fn add_encode_and_save_into_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    vector_ptr: LocalId,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    inner: &IntermediateType,
) {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (storage_load, _) = storage_load_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));
    let derive_dyn_array_slot_fn =
        RuntimeFunction::DeriveDynArraySlot.get(module, Some(compilation_ctx));

    // Locals
    let elem_slot_ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    // Stack size of the inner type
    let stack_size = inner.stack_data_size() as i32;

    // Element size in storage
    let elem_size = field_size(inner, compilation_ctx) as i32;

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
    let bytes_in_slot_offset = module.locals.add(ValType::I32);

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
                add_delete_field_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_slot_ptr,
                    inner,
                    elem_size,
                    bytes_in_slot_offset,
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

    // Reset the aux locals for the next loop
    // This loop encodes and saves the vector in memory to the storage.
    builder.i32_const(0).local_set(i);
    builder.i32_const(0).local_set(bytes_in_slot_offset);

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

                // If we have written the whole slot, save to storage and calculate the next slot
                loop_
                    .local_get(bytes_in_slot_offset)
                    .i32_const(elem_size)
                    .binop(BinaryOp::I32Add)
                    .i32_const(32)
                    .binop(BinaryOp::I32GtS)
                    .if_else(
                        None,
                        |then| {
                            // Save previous slot to storage
                            then.local_get(elem_slot_ptr)
                                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                                .call(storage_cache);

                            // Wipe the data so we can fill it with new data
                            then.i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                                .i32_const(0)
                                .i32_const(32)
                                .memory_fill(compilation_ctx.memory_id);

                            // Next slot
                            then.local_get(elem_slot_ptr)
                                .call(next_slot_fn)
                                .local_set(elem_slot_ptr);

                            // Set the written bytes in slot to the element size
                            then.i32_const(elem_size).local_set(bytes_in_slot_offset);
                        },
                        |else_| {
                            // Increment the written bytes in slot by the element size
                            else_
                                .local_get(bytes_in_slot_offset)
                                .i32_const(elem_size)
                                .binop(BinaryOp::I32Add)
                                .local_set(bytes_in_slot_offset);
                        },
                    );

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
                add_encode_intermediate_type_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_slot_ptr,
                    owner_ptr,
                    inner,
                    bytes_in_slot_offset,
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
}

/// Adds the instructions to encode and save a vector (as a field in a struct) into storage.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `data_ptr` - pointer to the memory region where the vector data will be written
/// `slot_ptr` - pointer to the vector header slot
/// `owner_ptr` - pointer to the owner struct id.
/// `inner` - inner type of the vector
pub fn add_read_and_decode_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    data_ptr: LocalId,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    inner: &IntermediateType,
) {
    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Locals
    let len = module.locals.add(ValType::I32);
    let elem_slot_ptr = module.locals.add(ValType::I32);
    let elem_data_ptr = module.locals.add(ValType::I32);

    // Allocate 32 bytes for the element slot
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(elem_slot_ptr);

    // Wipe the data so we write on it safely
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(0)
        .i32_const(32)
        .memory_fill(compilation_ctx.memory_id);

    // Load vector header slot data
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    // Load the length of the vector
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
        .local_set(len);

    // Stack size of the inner type
    let stack_size = inner.stack_data_size() as i32;

    // Element size in STORAGE
    let elem_size = field_size(inner, compilation_ctx) as i32;

    // Allocate memory for the vector and write the header data
    IVector::allocate_vector_with_header(builder, compilation_ctx, data_ptr, len, len, stack_size);

    // Iterate through the vector reading and decoding the elements from the storage.
    builder.block(None, |block| {
        let block_id = block.id();

        // Check if length == 0, if so skip the rest of the instructions.
        block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(block_id);

        // Calculate the slot of the first element
        block
            .local_get(slot_ptr)
            .i32_const(32)
            .local_get(elem_slot_ptr)
            .call(native_keccak);

        // Load the first slot data
        block
            .local_get(elem_slot_ptr)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);

        // Read the elements from the vector
        block.block(None, |iblock| {
            let iblock_id = iblock.id();

            let i = module.locals.add(ValType::I32);
            let read_bytes_in_slot = module.locals.add(ValType::I32);

            // Set the aux locals to 0 to start the loop
            iblock.i32_const(0).local_set(i);
            iblock.i32_const(0).local_set(read_bytes_in_slot);
            iblock.loop_(None, |loop_| {
                let loop_id = loop_.id();

                loop_
                    .local_get(read_bytes_in_slot)
                    .i32_const(elem_size)
                    .binop(BinaryOp::I32Add)
                    .i32_const(32)
                    .binop(BinaryOp::I32GtS)
                    .if_else(
                        None,
                        |then| {
                            // Calculate next slot to read from
                            then.local_get(elem_slot_ptr)
                                .call(next_slot_fn)
                                .local_tee(elem_slot_ptr);

                            // Load next slot data
                            then.i32_const(DATA_SLOT_DATA_PTR_OFFSET).call(storage_load);

                            then.i32_const(elem_size).local_set(read_bytes_in_slot);
                        },
                        |else_| {
                            else_
                                .local_get(read_bytes_in_slot)
                                .i32_const(elem_size)
                                .binop(BinaryOp::I32Add)
                                .local_set(read_bytes_in_slot);
                        },
                    );

                // Decode the element and store it at elem_data_ptr
                add_decode_intermediate_type_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_data_ptr,
                    elem_slot_ptr,
                    owner_ptr,
                    inner,
                    read_bytes_in_slot,
                );

                // Destination address of the element in memory
                loop_.vec_elem_ptr(data_ptr, i, stack_size);

                // Get the decoded element
                loop_.local_get(elem_data_ptr);

                // If the element is not heap, load the value from the intermediate pointer
                if inner.is_stack_type() {
                    loop_.load(
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
                };

                // Store the decoded element at data_ptr + i * stack_size
                loop_.store(
                    compilation_ctx.memory_id,
                    if stack_size == 8 {
                        StoreKind::I64 { atomic: false }
                    } else {
                        StoreKind::I32 { atomic: false }
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // If we reach the last element, we exit
                loop_
                    .local_get(i)
                    .local_get(len)
                    .i32_const(1)
                    .binop(BinaryOp::I32Sub)
                    .binop(BinaryOp::I32Eq)
                    .br_if(iblock_id);

                // Else, increment i and continue the loop
                loop_
                    .local_get(i)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_set(i)
                    .br(loop_id);
            });
        });
    });
}

/// Adds the instructions to encode and write an intermediate type to the storage slot.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `slot_ptr` - storage's slot where the data will be saved
/// `owner_ptr` - pointer to the owner struct id.
/// `itype` - intermediate type to be encoded
/// `written_bytes_in_slot` - number of bytes already written in the slot.
/// `is_field` - whether the type is a field from a struct or not.
///
/// Expects a pointer to the element in memory in stack
#[allow(clippy::too_many_arguments)]
pub fn add_encode_intermediate_type_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    itype: &IntermediateType,
    written_bytes_in_slot: LocalId,
    is_field: bool,
) {
    // Locals
    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);

    // Stack and storage size of the type
    let stack_size = itype.stack_data_size() as i32;
    let storage_size = field_size(itype, compilation_ctx) as i32;

    // Runtime functions
    let get_struct_id_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));

    match itype {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64 => {
            let (val, load_kind, swap_fn) = if stack_size == 8 {
                let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None);
                (val_64, LoadKind::I64 { atomic: false }, swap_fn)
            } else {
                let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
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
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .local_get(written_bytes_in_slot)
                .binop(BinaryOp::I32Sub)
                .binop(BinaryOp::I32Add)
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
            let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

            // Slot data plus offset as dest ptr
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .local_get(written_bytes_in_slot)
                .binop(BinaryOp::I32Sub)
                .binop(BinaryOp::I32Add);

            // Transform to BE
            builder.call(swap_fn);
        }
        IntermediateType::IU256 => {
            let swap_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

            // Slot data plus offset as dest ptr (offset should be zero because data is already
            // 32 bytes in size)
            builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

            // Transform to BE
            builder.call(swap_fn);
        }
        IntermediateType::IAddress | IntermediateType::ISigner => {
            // We need to swap values before copying because memory copy takes dest pointer
            // first
            builder.local_set(val_32);
            // Load the memory address

            // Slot data plus offset as dest ptr
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .local_get(written_bytes_in_slot)
                .binop(BinaryOp::I32Sub)
                .binop(BinaryOp::I32Add);

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
            let child_struct = compilation_ctx
                .get_struct_by_intermediate_type(itype)
                .expect("struct not found");

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

                // Reset written bytes counter for the child struct encoding
                builder.i32_const(0).local_set(written_bytes_in_slot);

                // Recursively encode and store the child struct
                add_encode_and_save_into_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    child_struct_ptr,
                    child_struct_slot_ptr,
                    None,
                    itype,
                    written_bytes_in_slot,
                );

                // After encoding the child struct, we need to store its UID in the
                // parent struct's data so the parent can reference the child.
                // The UID takes exactly 32 bytes (one full slot).

                // Update written bytes counter to reflect the 32-byte UID
                builder.i32_const(32).local_set(written_bytes_in_slot);

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
                    Some(owner_ptr),
                    itype,
                    written_bytes_in_slot,
                );
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
                owner_ptr,
                itype,
                written_bytes_in_slot,
            );
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
            );

            builder.i32_const(32).local_set(written_bytes_in_slot);
        }
        e => todo!("{e:?}"),
    };
}

/// Adds the instructions to decode and read an intermediate type from the storage slot.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `data_ptr` - pointer to where data is written
/// `slot_ptr` - storage's slot where the data is read
/// `owner_ptr` - pointer to the owner struct id.
/// `itype` - intermediate type to be decoded
/// `read_bytes_in_slot` - number of bytes already read in the slot.
#[allow(clippy::too_many_arguments)]
pub fn add_decode_intermediate_type_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    data_ptr: LocalId,
    slot_ptr: LocalId,
    owner_ptr: LocalId,
    itype: &IntermediateType,
    read_bytes_in_slot: LocalId,
) {
    // Stack and storage size of the type
    let stack_size = itype.stack_data_size() as i32;
    let storage_size = field_size(itype, compilation_ctx) as i32;

    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);

    // Runtime functions
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));

    match itype {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64 => {
            let (store_kind, swap_fn) = if stack_size == 8 {
                (
                    StoreKind::I64 { atomic: false },
                    RuntimeFunction::SwapI64Bytes.get(module, None),
                )
            } else {
                (
                    StoreKind::I32 { atomic: false },
                    RuntimeFunction::SwapI32Bytes.get(module, None),
                )
            };

            let load_kind = match storage_size {
                1 => LoadKind::I32_8 {
                    kind: ExtendedLoad::ZeroExtend,
                },
                2 => LoadKind::I32_16 {
                    kind: ExtendedLoad::ZeroExtend,
                },
                4 => LoadKind::I32 { atomic: false },
                8 => LoadKind::I64 { atomic: false },
                _ => panic!("invalid element size {storage_size} for type {itype:?}"),
            };

            // Allocate memory to write the decoded value
            builder
                .i32_const(stack_size)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr);

            // Load and swap the value
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .local_get(read_bytes_in_slot)
                .binop(BinaryOp::I32Sub)
                .load(
                    compilation_ctx.memory_id,
                    load_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .call(swap_fn);

            // If the size is less than 8 bytes we need to shift before saving
            if storage_size == 1 {
                builder.i32_const(24).binop(BinaryOp::I32ShrU);
            } else if storage_size == 2 {
                builder.i32_const(16).binop(BinaryOp::I32ShrU);
            }

            // Store the swapped value at #data_ptr
            builder.store(
                compilation_ctx.memory_id,
                store_kind,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU128 => {
            let copy_fn = RuntimeFunction::CopyU128.get(module, Some(compilation_ctx));
            let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

            // Copy 16 bytes from the slot data pointer (plus offset)
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .local_get(read_bytes_in_slot)
                .binop(BinaryOp::I32Sub)
                .call(copy_fn)
                .local_set(data_ptr);

            // Transform it to LE
            builder
                .local_get(data_ptr)
                .local_get(data_ptr)
                .call(swap_fn);
        }
        IntermediateType::IU256 => {
            let copy_fn = RuntimeFunction::CopyU256.get(module, Some(compilation_ctx));
            let swap_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

            // Copy 32 bytes from the slot data pointer
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(copy_fn)
                .local_set(data_ptr);

            // Transform it to LE
            builder
                .local_get(data_ptr)
                .local_get(data_ptr)
                .call(swap_fn);
        }
        IntermediateType::IAddress | IntermediateType::ISigner => {
            // Allocate memory for the address
            builder
                .i32_const(32)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr);

            // Add 12 to the offset to write the last 20 bytes of the address
            builder.i32_const(12).binop(BinaryOp::I32Add);

            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .local_get(read_bytes_in_slot)
                .binop(BinaryOp::I32Sub)
                .binop(BinaryOp::I32Add);

            // Number of bytes to copy
            builder.i32_const(20);

            // Copy the chunk of memory
            builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
            // ========================================================================
            // Handle Nested Struct Decoding
            // ========================================================================
            // This section handles decoding of nested structs from storage.
            // The behavior differs based on whether the child struct has the 'key' ability:
            // - If child has 'key': read UID from parent, calculate child slot, decode child
            // - If child has no 'key': decode child directly from current slot (flattened)

            // Get base definition by (module_id, index)
            let child_struct = compilation_ctx
                .get_struct_by_intermediate_type(itype)
                .expect("struct not found");

            if child_struct.has_key {
                // ====================================================================
                // CHILD STRUCT WITH KEY - Decode from Separate Object
                // ====================================================================
                // When a child struct has the 'key' ability, it is stored as a separate
                // object in storage. To decode it, we need to:
                // 1. Read the child struct UID from the parent's data
                // 2. Calculate the child struct's storage slot
                // 3. Decode the child struct from its dedicated slot
                // 4. Return the decoded child struct

                // Retrieve the 32-byte UID of the child struct from the current slot.
                // This UID, saved during the encoding process in the parent struct, links the child struct to its parent.

                // Allocate memory for the child struct UID (32 bytes)
                let child_struct_id_ptr = module.locals.add(ValType::I32);
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(child_struct_id_ptr);

                // Load the child struct UID from storage
                builder
                    .local_get(slot_ptr)
                    .local_get(child_struct_id_ptr)
                    .call(storage_load);

                // Calculate the child struct's storage slot
                // child_struct_slot = keccak256(child_struct_id || keccak256(owner || 0))
                builder
                    .local_get(owner_ptr)
                    .local_get(child_struct_id_ptr)
                    .call(write_object_slot_fn);

                // Allocate memory for the child struct slot and copy the calculated
                // slot data to avoid overwriting during recursive decoding.

                let child_struct_slot_ptr = module.locals.add(ValType::I32);
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(child_struct_slot_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Reset read bytes counter for the child struct decoding
                builder.i32_const(0).local_set(read_bytes_in_slot);

                // Recursively decode the child struct
                // The calculated slot is used as the base slot
                // The child struct UID is used as the struct UID
                // The parent struct UID is used as the struct owner
                let child_struct_ptr = add_read_and_decode_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    child_struct_slot_ptr,
                    Some(child_struct_id_ptr),
                    owner_ptr,
                    itype,
                    read_bytes_in_slot,
                );

                // Update read bytes counter to reflect the 32-byte UID we consumed
                builder.i32_const(32).local_set(read_bytes_in_slot);

                // Set the decoded child struct as the result
                builder.local_get(child_struct_ptr).local_set(data_ptr);
            } else {
                // ====================================================================
                // CHILD STRUCT WITHOUT KEY - Decode from Flattened Data
                // ====================================================================
                // When a child struct doesn't have the 'key' ability, it was stored
                // flattened within the parent struct's data. We can decode it directly
                // from the current slot without needing to calculate separate storage.

                // Decode the child struct directly from the current slot
                // The child struct's fields are stored inline within the parent's data
                let child_struct_ptr = add_read_and_decode_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    slot_ptr,
                    None,
                    owner_ptr,
                    itype,
                    read_bytes_in_slot,
                );

                // Set the decoded child struct as the result
                builder.local_get(child_struct_ptr).local_set(data_ptr);
            }
        }
        IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
            let enum_ptr = add_read_and_decode_storage_enum_instructions(
                module,
                builder,
                compilation_ctx,
                slot_ptr,
                owner_ptr,
                itype,
                read_bytes_in_slot,
            );

            // Set the decoded enum as the result
            builder.local_get(enum_ptr).local_set(data_ptr);
        }
        IntermediateType::IVector(inner_) => {
            add_read_and_decode_storage_vector_instructions(
                module,
                builder,
                compilation_ctx,
                data_ptr,
                slot_ptr,
                owner_ptr,
                inner_,
            );
        }
        _ => todo!(),
    };
}

/// Return the storage-encoded field size in bytes
pub fn field_size(field: &IntermediateType, compilation_ctx: &CompilationContext) -> u32 {
    match field {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => 1,
        IntermediateType::IU16 => 2,
        IntermediateType::IU32 => 4,
        IntermediateType::IU64 => 8,
        IntermediateType::IU128 => 16,
        IntermediateType::IU256 => 32,
        IntermediateType::IAddress | IntermediateType::ISigner => 20,
        // Dynamic data occupies the whole slot, but the data is saved somewhere else
        IntermediateType::IVector(_) => 32,
        field if field.is_uid_or_named_id(compilation_ctx) => 32,

        // Structs default to size 0 since their size depends on whether their fields are dynamic or static.
        // The store function will handle this. If a struct has the 'key' ability, it at least occupies 32 bytes for the UID.
        // The store function will manage the rest of the fields.
        IntermediateType::IGenericStructInstance {
            module_id, index, ..
        }
        | IntermediateType::IStruct {
            module_id, index, ..
        } => {
            let s = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .expect("struct not found");

            if s.has_key { 32 } else { 0 }
        }
        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            panic!("found reference inside struct")
        }
        IntermediateType::ITypeParameter(_) => {
            panic!("cannot know the field size of a type parameter");
        }
    }
}

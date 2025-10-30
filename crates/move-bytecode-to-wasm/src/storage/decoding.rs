//! This module implements the logic to read/decode data from storage slots.
//!
//! The decoding layout mirrors Solidity's storage layout.
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET},
    hostio::host_functions::{native_keccak256, storage_load_bytes32},
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, vector::IVector},
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::encoding::field_size;

/// Emits WASM instructions that read a struct from storage and decode it into
/// its in-memory representation.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `slot_ptr`: Local pointing to the base storage slot of the struct.
/// - `struct_id_ptr`: Optional local to the struct UID (required if the struct has `key`).
/// - `owner_ptr`: Local pointing to the owner struct UID (used for nested keyed objects).
/// - `itype`: Intermediate type of the struct to decode.
/// - `read_bytes_in_slot`: Local used as a running counter of bytes already read in the current slot.
///
/// Returns:
/// - LocalId to the heap pointer of the decoded struct in linear memory.
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
    let accumulate_or_advance_slot_fn =
        RuntimeFunction::AccumulateOrAdvanceSlot.get(module, Some(compilation_ctx));

    // Get the IStruct representation
    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap_or_else(|_| {
            panic!("there was an error decoding a struct for storage, found {itype:?}")
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

            builder
                .local_get(slot_ptr)
                .local_get(read_bytes_in_slot)
                .i32_const(field_size)
                .i32_const(1)
                .call(accumulate_or_advance_slot_fn)
                .local_set(read_bytes_in_slot);

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

/// Emits WASM instructions that read a tagged enum from storage and decode the
/// active variant into its in-memory representation.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `slot_ptr`: Local pointing to the base storage slot of the enum.
/// - `owner_ptr`: Local pointing to the owner struct UID (for nested keyed objects).
/// - `itype`: Intermediate type of the enum to decode.
/// - `read_bytes_in_slot`: Local used as a running counter of bytes already read in the current slot.
///
/// Returns:
/// - LocalId to the heap pointer of the decoded enum in linear memory.
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
    // Runtime functions
    let accumulate_or_advance_slot_fn =
        RuntimeFunction::AccumulateOrAdvanceSlot.get(module, Some(compilation_ctx));

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
        .add_slot_data_ptr_plus_offset(read_bytes_in_slot)
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

            block
                .local_get(slot_ptr)
                .local_get(read_bytes_in_slot)
                .i32_const(field_size)
                .i32_const(1)
                .call(accumulate_or_advance_slot_fn)
                .local_set(read_bytes_in_slot);

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

/// Emits WASM instructions to read a vector from storage and decode its
/// elements into a contiguous in-memory vector representation.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `data_ptr`: Local that will receive the heap pointer to the resulting vector.
/// - `slot_ptr`: Local pointing to the vector header slot in storage.
/// - `owner_ptr`: Local pointing to the owner struct UID (for nested keyed objects).
/// - `inner`: Intermediate type of the vector elements.
///
/// Returns:
/// - None. Writes the allocated vector pointer into `data_ptr` and fills its contents.
#[allow(clippy::too_many_arguments)]
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
    let accumulate_or_advance_slot_fn =
        RuntimeFunction::AccumulateOrAdvanceSlot.get(module, Some(compilation_ctx));

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

    // Iterate through the vector reading and decoding the elements from storage.
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
                    .local_get(elem_slot_ptr)
                    .local_get(read_bytes_in_slot)
                    .i32_const(elem_size)
                    .i32_const(1)
                    .call(accumulate_or_advance_slot_fn)
                    .local_set(read_bytes_in_slot);

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

/// Emits WASM instructions to read a value of an intermediate type from a storage
/// slot and decode it into its in-memory representation.
///
/// Arguments:
/// - `module`: Walrus module being built.
/// - `builder`: Instruction sequence builder to append to.
/// - `compilation_ctx`: Shared compilation context (types, memory, helpers).
/// - `data_ptr`: Local that will receive the heap pointer (or stack cell) with the decoded value.
/// - `slot_ptr`: Local pointing to the base storage slot to read from.
/// - `owner_ptr`: Local pointing to the owner struct UID (relevant for keyed objects).
/// - `itype`: Intermediate type to decode.
/// - `read_bytes_in_slot`: Local used as a running counter of bytes already read in the current slot.
///
/// Returns:
/// - None. Writes the resulting pointer/value location into `data_ptr`.
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
                .add_slot_data_ptr_plus_offset(read_bytes_in_slot)
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
                .add_slot_data_ptr_plus_offset(read_bytes_in_slot)
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
                .add_slot_data_ptr_plus_offset(read_bytes_in_slot)
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

            builder.add_slot_data_ptr_plus_offset(read_bytes_in_slot);

            // Number of bytes to copy
            builder.i32_const(20);

            // Copy the chunk of memory
            builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
            // Get base definition by (module_id, index)
            let child_struct = compilation_ctx
                .get_struct_by_intermediate_type(itype)
                .expect("struct not found");

            if child_struct.has_key {
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

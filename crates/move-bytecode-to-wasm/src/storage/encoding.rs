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
    compilation_context::ExternalModuleData,
    data::{
        DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET,
        DATA_STORAGE_OBJECT_OWNER_OFFSET,
    },
    hostio::host_functions::{native_keccak256, storage_cache_bytes32, storage_load_bytes32},
    runtime::RuntimeFunction,
    translation::intermediate_types::vector::IVector,
    translation::intermediate_types::{
        IntermediateType,
        heap_integers::{IU128, IU256},
        structs::IStruct,
    },
    vm_handled_types::{VmHandledType, uid::Uid},
    wasm_builder_extensions::WasmBuilderExtension,
};

/// Adds the instructions to encode and save into storage an specific struct.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `struct_ptr` - pointer to the struct to be encoded
/// `slot_ptr` - storage's slot where the data will be saved
/// `struct_` - structural information of the struct to be encoded and saved
/// `written_bytes_in_slot` - number of bytes already written in the slot. This will be != 0 if
/// this function is recusively called to save a struct inside another struct.
///
/// # Returns
/// The written_bytes_in_slot value. Used to update the caller of the recursive call
pub fn add_encode_and_save_into_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    struct_ptr: LocalId,
    slot_ptr: LocalId,
    struct_: &IStruct,
    written_bytes_in_slot: u32,
) -> u32 {
    let (storage_cache, _) = storage_cache_bytes32(module);
    #[cfg(feature = "inject-host-debug-fns")]
    let (print_i32, print_i64, print_memory_from, print_address, print_separator, print_u128) = {
        crate::inject_debug_fns(module);
        crate::declare_host_debug_functions!(module)
    };
    // Locals
    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);

    let mut written_bytes_in_slot = written_bytes_in_slot;
    for (index, field) in struct_.fields.iter().enumerate() {
        let field_size = field_size(field, compilation_ctx);
        if written_bytes_in_slot + field_size > 32 {
            // Save previous slot (maybe not needed...)
            builder
                .local_get(slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_cache);

            // Wipe the data so we can fill it with new data
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(0)
                .i32_const(32)
                .memory_fill(compilation_ctx.memory_id);

            let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));
            builder
                .local_get(slot_ptr)
                .call(next_slot_fn)
                .local_set(slot_ptr);

            written_bytes_in_slot = field_size;
        } else {
            written_bytes_in_slot += field_size;
        }

        // Load field's intermediate pointer
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let (val, load_kind, swap_fn) = if field.stack_data_size() == 8 {
                    let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None);
                    (val_64, LoadKind::I64 { atomic: false }, swap_fn)
                } else {
                    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
                    (val_32, LoadKind::I32 { atomic: false }, swap_fn)
                };

                builder
                    .load(
                        compilation_ctx.memory_id,
                        load_kind,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(val);

                // Convert the value to big endian
                builder.call(swap_fn).local_set(val);

                // We need to shift the swapped bytes to the right because WASM is little endian. If we try
                // to write a 16 bits number contained in a 32 bits number, without shifting, it will write
                // the zeroed part.
                // This only needs to be done for 32 bits (4 bytes) numbers
                if field.stack_data_size() == 4 {
                    if field_size == 1 {
                        builder
                            .local_get(val)
                            .i32_const(24)
                            .binop(BinaryOp::I32ShrU)
                            .local_set(val);
                    } else if field_size == 2 {
                        builder
                            .local_get(val)
                            .i32_const(16)
                            .binop(BinaryOp::I32ShrU)
                            .local_set(val);
                    }
                }

                let store_kind = if field_size == 1 {
                    StoreKind::I32_8 { atomic: false }
                } else if field_size == 2 {
                    StoreKind::I32_16 { atomic: false }
                } else if field_size == 4 {
                    StoreKind::I32 { atomic: false }
                } else {
                    StoreKind::I64 { atomic: false }
                };

                // Save the value in slot data
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .local_get(val)
                    .store(
                        compilation_ctx.memory_id,
                        store_kind,
                        MemArg {
                            align: 0,
                            offset: 32 - written_bytes_in_slot,
                        },
                    );
            }
            IntermediateType::IU128 => {
                let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

                // Slot data plus offset as dest ptr
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .i32_const(32 - written_bytes_in_slot as i32)
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
                let tmp = module.locals.add(ValType::I32);
                builder.local_set(tmp);
                // Load the memory address

                // Slot data plus offset as dest ptr
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .i32_const(32 - written_bytes_in_slot as i32)
                    .binop(BinaryOp::I32Add);

                // Grab the last 20 bytes of the address
                builder.local_get(tmp).i32_const(12).binop(BinaryOp::I32Add);

                // Amount of bytes to copy
                builder.i32_const(20);

                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            }
            IntermediateType::IStruct { module_id, index } => {
                let child_struct = compilation_ctx
                    .get_user_data_type_by_index(module_id, *index)
                    .unwrap();

                // The struct ptr
                let tmp = module.locals.add(ValType::I32);
                builder.local_set(tmp);

                written_bytes_in_slot = add_encode_and_save_into_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    tmp,
                    slot_ptr,
                    child_struct,
                    written_bytes_in_slot,
                );
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let child_struct = compilation_ctx
                    .get_user_data_type_by_index(module_id, *index)
                    .unwrap();
                let child_struct = child_struct.instantiate(types);

                // The struct ptr
                let tmp = module.locals.add(ValType::I32);
                builder.local_set(tmp);

                written_bytes_in_slot = add_encode_and_save_into_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    tmp,
                    slot_ptr,
                    &child_struct,
                    written_bytes_in_slot,
                );
            }
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
                ..
            } if Uid::is_vm_type(module_id, identifier) => {
                let tmp = module.locals.add(ValType::I32);

                // The UID struct has the following form
                //
                // UID { id: ID { bytes: <bytes> } }
                //
                // At this point we have in stack a pointer to field we are processing. The
                // field's value is a pointer to the ID struct.
                //
                // The first load instruction puts in stack the pointer to the ID struct
                // The second load instruction loads the ID's bytes field pointer
                //
                // At the end of the load chain we point to the 32 bytes holding the data
                builder
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(tmp);

                // Load the memory address
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .local_get(tmp)
                    .i32_const(32);

                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            }
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
                types,
            } => {
                let external_data = compilation_ctx
                    .get_external_module_data(module_id, identifier, types)
                    .unwrap();

                match external_data {
                    ExternalModuleData::Struct(struct_) => {
                        // The struct ptr
                        let tmp = module.locals.add(ValType::I32);
                        builder.local_set(tmp);

                        written_bytes_in_slot =
                            add_encode_and_save_into_storage_struct_instructions(
                                module,
                                builder,
                                compilation_ctx,
                                tmp,
                                slot_ptr,
                                &struct_,
                                written_bytes_in_slot,
                            );
                    }
                    ExternalModuleData::Enum(enum_) => {
                        if !enum_.is_simple {
                            panic!(
                                "cannot abi pack enum, it contains at least one variant with fields"
                            );
                        }
                        todo!();
                    }
                }
            }
            IntermediateType::IVector(inner) => {
                let vector_ptr = module.locals.add(ValType::I32);
                builder.local_set(vector_ptr);

                add_encode_and_save_into_storage_vector_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    vector_ptr,
                    slot_ptr,
                    inner,
                );

                // TODO: Do we need this?
                written_bytes_in_slot = 32; // Vector header always takes 32 bytes
            }
            e => todo!("{e:?}"),
        };
    }

    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);

    written_bytes_in_slot
}

/// Adds the instructions to read, decode from storage and build in memory a structure.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `slot_ptr` - storage's slot where the data will be saved
/// `struct_` - structural information of the struct to be encoded and saved
/// `reading_nested_struct` - if true, this function is called to read a nested struct inside
/// another struct.
///
/// # Returns
/// pointer where the read struct is allocated
pub fn add_read_and_decode_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    struct_: &IStruct,
    reading_nested_struct: bool,
    read_bytes_in_slot: u32,
) -> (LocalId, u32) {
    let (storage_load, _) = storage_load_bytes32(module);

    #[cfg(feature = "inject-host-debug-fns")]
    let (print_i32, print_i64, print_memory_from, print_address, print_separator, print_u128) = {
        crate::inject_debug_fns(module);
        crate::declare_host_debug_functions!(module)
    };

    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let field_ptr = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);
    let val_32 = module.locals.add(ValType::I32);

    // If we are reading an struct from the storage, means this struct has an owner and that owner
    // is saved in the DATA_STORAGE_OBJECT_OWNER_OFFSET piece of reserved memory. To be able to
    // know its owner when manipulating the reconstructed structure (for example for the saving the
    // changes in storage or transfering it) before its representation in memory, we save the owner
    // id
    if !reading_nested_struct {
        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .i32_const(32)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
    }

    // Allocate space for the struct
    builder
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(struct_ptr);

    // Load data from slot
    if !reading_nested_struct {
        builder
            .local_get(slot_ptr)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);
    }

    let mut read_bytes_in_slot = read_bytes_in_slot;
    for (index, field) in struct_.fields.iter().enumerate() {
        let field_size = field_size(field, compilation_ctx);
        if read_bytes_in_slot + field_size > 32 {
            let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));
            builder
                .local_get(slot_ptr)
                .call(next_slot_fn)
                .local_set(slot_ptr);

            // Load the slot data
            builder
                .local_get(slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_load);

            read_bytes_in_slot = field_size;
        } else {
            read_bytes_in_slot += field_size;
        }

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let data_size = field.stack_data_size();
                let (val, store_kind, swap_fn) = if data_size == 8 {
                    let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None);
                    (val_64, StoreKind::I64 { atomic: false }, swap_fn)
                } else {
                    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
                    (val_32, StoreKind::I32 { atomic: false }, swap_fn)
                };

                // Create a pointer for the value
                builder
                    .i32_const(data_size as i32)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Read the value from the slot
                let load_kind = match field_size {
                    1 => LoadKind::I32_8 {
                        kind: ExtendedLoad::ZeroExtend,
                    },
                    2 => LoadKind::I32_16 {
                        kind: ExtendedLoad::ZeroExtend,
                    },
                    4 => LoadKind::I32 { atomic: false },
                    8 => LoadKind::I64 { atomic: false },
                    _ => panic!("invalid field size {field_size} for type {field:?}"),
                };

                // Read the value and transform it to LE
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .load(
                        compilation_ctx.memory_id,
                        load_kind,
                        MemArg {
                            align: 0,
                            offset: 32 - read_bytes_in_slot,
                        },
                    )
                    .local_tee(val)
                    .call(swap_fn)
                    .local_set(val);

                // If the field size are less than 4 or 8 bytes we need to shift them before
                // saving
                if field_size == 1 {
                    builder
                        .local_get(val)
                        .i32_const(24)
                        .binop(BinaryOp::I32ShrU)
                        .local_set(val);
                } else if field_size == 2 {
                    builder
                        .local_get(val)
                        .i32_const(16)
                        .binop(BinaryOp::I32ShrU)
                        .local_set(val);
                }

                // Save it to the struct
                builder.local_get(val).store(
                    compilation_ctx.memory_id,
                    store_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU128 => {
                // Create a pointer for the value
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Source address (plus offset)
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .i32_const(32 - read_bytes_in_slot as i32)
                    .binop(BinaryOp::I32Add);

                // Number of bytes to copy
                builder.i32_const(IU128::HEAP_SIZE);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

                // Transform it to LE
                builder
                    .local_get(field_ptr)
                    .local_get(field_ptr)
                    .call(swap_fn);
            }
            IntermediateType::IU256 => {
                // Create a pointer for the value
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Source address (plus offset)
                builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                // Number of bytes to copy
                builder.i32_const(32);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                let swap_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

                // Transform it to LE
                builder
                    .local_get(field_ptr)
                    .local_get(field_ptr)
                    .call(swap_fn);
            }
            IntermediateType::IAddress | IntermediateType::ISigner => {
                // Create a pointer for the value
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Add 12 to the offset to write the last 20 bytes of the address
                builder.i32_const(12).binop(BinaryOp::I32Add);

                // Source address (plus offset)
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .i32_const(32 - read_bytes_in_slot as i32)
                    .binop(BinaryOp::I32Add);

                // Number of bytes to copy
                builder.i32_const(20);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            }
            IntermediateType::IStruct { module_id, index } => {
                let child_struct = compilation_ctx
                    .get_user_data_type_by_index(module_id, *index)
                    .unwrap();

                // Read the child struct
                let (child_struct_ptr, read_bytes) =
                    add_read_and_decode_storage_struct_instructions(
                        module,
                        builder,
                        compilation_ctx,
                        slot_ptr,
                        child_struct,
                        true,
                        read_bytes_in_slot,
                    );

                read_bytes_in_slot = read_bytes;

                builder.local_get(child_struct_ptr).local_set(field_ptr);
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => {
                let child_struct = compilation_ctx
                    .get_user_data_type_by_index(module_id, *index)
                    .unwrap();
                let child_struct = child_struct.instantiate(types);

                // Read the child struct
                let (child_struct_ptr, read_bytes) =
                    add_read_and_decode_storage_struct_instructions(
                        module,
                        builder,
                        compilation_ctx,
                        slot_ptr,
                        &child_struct,
                        true,
                        read_bytes_in_slot,
                    );

                read_bytes_in_slot = read_bytes;

                builder.local_get(child_struct_ptr).local_set(field_ptr);
            }
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
                ..
            } if Uid::is_vm_type(module_id, identifier) => {
                // Here we need to reconstruct the UID struct. To do that we first allocate 4 bytes
                // that will contain the pointer to the UID struct data
                //
                // After that we need to create the ID struct. So we allocate 4 bytes for the first
                // field's pointer, and 32 bytes that will hold the actual data.

                // Create a pointer for the value. This pointer will point to the struct ID
                builder
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_set(field_ptr);

                let id_struct_ptr = module.locals.add(ValType::I32);
                let id_field_ptr = module.locals.add(ValType::I32);

                // Recreate the ID struct

                // First, 4 bytes for the pointer that points to the ID
                builder
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_set(id_struct_ptr);

                // 32 bytes to save the actual id
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(id_field_ptr);

                // Source address (plus offset)
                builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                // Number of bytes to copy
                builder.i32_const(32);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Point the id_field_ptr to the data
                builder
                    .local_get(id_struct_ptr)
                    .local_get(id_field_ptr)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );

                // Write the field_ptr with the address of the ID struct
                builder.local_get(field_ptr).local_get(id_struct_ptr).store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
                types,
            } => {
                let external_data = compilation_ctx
                    .get_external_module_data(module_id, identifier, types)
                    .unwrap();

                match external_data {
                    ExternalModuleData::Struct(child_struct) => {
                        // Read the child struct
                        let (child_struct_ptr, read_bytes) =
                            add_read_and_decode_storage_struct_instructions(
                                module,
                                builder,
                                compilation_ctx,
                                slot_ptr,
                                &child_struct,
                                true,
                                read_bytes_in_slot,
                            );

                        read_bytes_in_slot = read_bytes;

                        builder.local_get(child_struct_ptr).local_set(field_ptr);
                    }
                    ExternalModuleData::Enum(_) => {
                        todo!();
                    }
                }
            }
            IntermediateType::IVector(inner) => {
                add_read_and_decode_storage_vector_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    field_ptr, // this local has not been set yet!
                    slot_ptr,
                    inner,
                );
            }
            _ => todo!(),
        };

        // Save the ptr value to the struct
        builder.local_get(struct_ptr).local_get(field_ptr).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );
    }

    (struct_ptr, read_bytes_in_slot)
}

/// Return the storage-encoded field size in bytes
pub fn field_size(field: &IntermediateType, compilation_ctx: &CompilationContext) -> u32 {
    match field {
        IntermediateType::IBool | IntermediateType::IU8 | IntermediateType::IEnum(_) => 1,
        IntermediateType::IU16 => 2,
        IntermediateType::IU32 => 4,
        IntermediateType::IU64 => 8,
        IntermediateType::IU128 => 16,
        IntermediateType::IU256 => 32,
        IntermediateType::IAddress | IntermediateType::ISigner => 20,
        // Dynamic data occupies the whole slot, but the data is saved somewhere else
        IntermediateType::IVector(_) => 32,

        // Structs are 0 because we don't know how much they will occupy, this depends on the
        // fields of the child struct, whether they are dynamic or static. The store function
        // called will take care of this.
        IntermediateType::IGenericStructInstance { .. } | IntermediateType::IStruct { .. } => 0,
        IntermediateType::IExternalUserData {
            module_id,
            identifier,
            ..
        } if Uid::is_vm_type(module_id, identifier) => 32,
        IntermediateType::IExternalUserData {
            module_id,
            identifier,
            types,
        } => match compilation_ctx
            .get_external_module_data(module_id, identifier, types)
            .unwrap()
        {
            ExternalModuleData::Struct(_) => 0,
            ExternalModuleData::Enum(_) => 1,
        },

        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            panic!("found reference inside struct")
        }
        IntermediateType::ITypeParameter(_) => {
            panic!("cannot know if a type parameter is dynamic, expected a concrete type");
        }
    }
}

/// Adds the instructions to encode and save a vector (as a field in a struct) into storage.
pub fn add_encode_and_save_into_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    vector_ptr: LocalId,      // struct_ptr.load(offset: i)
    slot_ptr: LocalId,        // pointer to the slot where the vector header will be stored
    inner: &IntermediateType, // vector inner type
) {
    #[cfg(feature = "inject-host-debug-fns")]
    let (print_i32, print_i64, print_memory_from, print_address, print_separator, print_u128) = {
        crate::inject_debug_fns(module);
        crate::declare_host_debug_functions!(module)
    };

    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Locals
    let elem_slot_ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);
    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);

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

    // Outer block: if the vector length is 0, we skip to the end
    builder.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if length == 0
        outer_block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(outer_block_id);

        // Stack size of the inner type
        let stack_size = inner.stack_data_size();

        // Element size in STORAGE
        // TODO: look out for structs, they have field size = 0.
        let elem_size = field_size(inner, compilation_ctx);

        // Calculate the slot of the first element
        outer_block
            .local_get(slot_ptr)
            .i32_const(32)
            .local_get(elem_slot_ptr)
            .call(native_keccak);

        outer_block.block(None, |inner_block| {
            let inner_block_id = inner_block.id();

            let i = module.locals.add(ValType::I32);
            let written_bytes_in_slot = module.locals.add(ValType::I32);

            // Set the aux locals to 0 to start the loop
            inner_block.i32_const(0).local_set(i);
            inner_block.i32_const(0).local_set(written_bytes_in_slot);

            inner_block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // If we have written the whole slot, save to storage and calculate the next slot
                loop_
                    .local_get(written_bytes_in_slot)
                    .i32_const(elem_size as i32)
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

                            then.i32_const(elem_size as i32)
                                .local_set(written_bytes_in_slot);
                        },
                        |else_| {
                            else_
                                .local_get(written_bytes_in_slot)
                                .i32_const(elem_size as i32)
                                .binop(BinaryOp::I32Add)
                                .local_set(written_bytes_in_slot);
                        },
                    );

                // ptr to the element in memory
                loop_.vec_elem_ptr(vector_ptr, i, stack_size as i32);

                match inner {
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

                        let store_kind = if elem_size == 1 {
                            StoreKind::I32_8 { atomic: false }
                        } else if elem_size == 2 {
                            StoreKind::I32_16 { atomic: false }
                        } else if elem_size == 4 {
                            StoreKind::I32 { atomic: false }
                        } else {
                            StoreKind::I64 { atomic: false }
                        };

                        loop_
                            .load(
                                compilation_ctx.memory_id,
                                load_kind,
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .call(swap_fn);

                        // We need to shift the swapped bytes to the right because WASM is little endian. If we try
                        // to write a 16 bits number contained in a 32 bits number, without shifting, it will write
                        // the zeroed part.
                        // This only needs to be done for 32 bits (4 bytes) numbers
                        if stack_size == 4 {
                            if elem_size == 1 {
                                loop_.i32_const(24).binop(BinaryOp::I32ShrU);
                            } else if elem_size == 2 {
                                loop_.i32_const(16).binop(BinaryOp::I32ShrU);
                            }
                        }

                        loop_.local_set(val);

                        // Save the value in slot data
                        // Calculate address: DATA_SLOT_DATA_PTR_OFFSET + (32 - written_bytes_in_slot)
                        loop_
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .i32_const(32)
                            .binop(BinaryOp::I32Add)
                            .local_get(written_bytes_in_slot)
                            .binop(BinaryOp::I32Sub)
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
                        let swap_fn =
                            RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

                        // Load the pointer to the u128 element
                        loop_.load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                        // Slot data plus offset as dest ptr
                        loop_
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .i32_const(32)
                            .binop(BinaryOp::I32Add)
                            .local_get(written_bytes_in_slot)
                            .binop(BinaryOp::I32Sub);

                        loop_.call(swap_fn);
                    }
                    IntermediateType::IU256 => {
                        let swap_fn =
                            RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

                        // Load the pointer to the u256 element
                        loop_.load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                        // Slot data plus offset as dest ptr (offset should be zero because data is already
                        // 32 bytes in size)
                        loop_.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                        // Transform to BE
                        loop_.call(swap_fn);
                    }
                    IntermediateType::IAddress | IntermediateType::ISigner => {
                        // Load the pointer to the address element
                        // Currently this points to a 32 bytes memory chunk, where the first 12 bytes are zeroed.
                        loop_
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_set(val_32);

                        // Load the memory address
                        // Slot data plus offset as dest ptr
                        loop_
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .i32_const(32)
                            .binop(BinaryOp::I32Add)
                            .local_get(written_bytes_in_slot)
                            .binop(BinaryOp::I32Sub);

                        // Skip the first 12 bytes (zeros) of the address memory
                        loop_
                            .local_get(val_32)
                            .i32_const(12)
                            .binop(BinaryOp::I32Add);

                        // Amount of bytes to copy
                        loop_.i32_const(20);
                        loop_.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
                    }
                    IntermediateType::IStruct { module_id, index } => {
                        let inner_data_ptr = module.locals.add(ValType::I32);

                        let child_struct = compilation_ctx
                            .get_user_data_type_by_index(module_id, *index)
                            .unwrap();

                        // Load the struct pointer
                        loop_
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_set(inner_data_ptr);

                        // add_encode_and_save_into_storage_struct_instructions will modify the slot pointer so we know where to continue once this function returns.
                        let written_bytes_in_slot_ =
                            add_encode_and_save_into_storage_struct_instructions(
                                module,
                                loop_,
                                compilation_ctx,
                                inner_data_ptr,
                                elem_slot_ptr,
                                child_struct,
                                0,
                            );
                    }
                    // IntermediateType::IGenericStructInstance {
                    //     module_id,
                    //     index,
                    //     types,
                    // } => {
                    //     let child_struct = compilation_ctx
                    //         .get_user_data_type_by_index(module_id, *index)
                    //         .unwrap();
                    //     let child_struct = child_struct.instantiate(types);

                    //     // The struct ptr
                    //     let tmp = module.locals.add(ValType::I32);
                    //     loop_.local_set(tmp);

                    //     written_bytes_in_slot = add_encode_and_save_into_storage_struct_instructions(
                    //         module,
                    //         loop_,
                    //         compilation_ctx,
                    //         tmp,
                    //         elem_slot_ptr, // TODO: check if this is correct!
                    //         &child_struct,
                    //         written_bytes_in_slot,
                    //     );
                    // }
                    // IntermediateType::IExternalUserData {
                    //     module_id,
                    //     identifier,
                    //     ..
                    // } if Uid::is_vm_type(module_id, identifier) => {
                    //     let tmp = module.locals.add(ValType::I32);

                    //     // The UID struct has the following form
                    //     //
                    //     // UID { id: ID { bytes: <bytes> } }
                    //     //
                    //     // At this point we have in stack a pointer to field we are processing. The
                    //     // field's value is a pointer to the ID struct.
                    //     //
                    //     // The first load instruction puts in stack the pointer to the ID struct
                    //     // The second load instruction loads the ID's bytes field pointer
                    //     //
                    //     // At the end of the load chain we point to the 32 bytes holding the data
                    //     loop_
                    //         .load(
                    //             compilation_ctx.memory_id,
                    //             LoadKind::I32 { atomic: false },
                    //             MemArg {
                    //                 align: 0,
                    //                 offset: 0,
                    //             },
                    //         )
                    //         .load(
                    //             compilation_ctx.memory_id,
                    //             LoadKind::I32 { atomic: false },
                    //             MemArg {
                    //                 align: 0,
                    //                 offset: 0,
                    //             },
                    //         )
                    //         .local_set(tmp);

                    //     // Load the memory address
                    //     loop_
                    //         .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    //         .local_get(tmp)
                    //         .i32_const(32);

                    //     loop_.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
                    // }
                    // IntermediateType::IExternalUserData {
                    //     module_id,
                    //     identifier,
                    //     types,
                    // } => {
                    //     let external_data = compilation_ctx
                    //         .get_external_module_data(module_id, identifier, types)
                    //         .unwrap();

                    //     match external_data {
                    //         ExternalModuleData::Struct(struct_) => {
                    //             // The struct ptr
                    //             let tmp = module.locals.add(ValType::I32);
                    //             loop_.local_set(tmp);

                    //             written_bytes_in_slot =
                    //                 add_encode_and_save_into_storage_struct_instructions(
                    //                     module,
                    //                     loop_,
                    //                     compilation_ctx,
                    //                     tmp,
                    //                     elem_slot_ptr, // TODO: check if this is correct!
                    //                     &struct_,
                    //                     written_bytes_in_slot,
                    //                 );
                    //         }
                    //         ExternalModuleData::Enum(enum_) => {
                    //             if !enum_.is_simple {
                    //                 panic!(
                    //                     "cannot abi pack enum, it contains at least one variant with fields"
                    //                 );
                    //             }
                    //             todo!();
                    //         }
                    //     }
                    // }
                    IntermediateType::IVector(inner_) => {
                        let inner_vector_ptr = module.locals.add(ValType::I32);
                        let inner_slot_ptr = module.locals.add(ValType::I32);

                        loop_
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_set(inner_vector_ptr);

                        loop_.local_get(elem_slot_ptr).local_set(inner_slot_ptr);

                        add_encode_and_save_into_storage_vector_instructions(
                            module,
                            loop_,
                            compilation_ctx,
                            inner_vector_ptr,
                            inner_slot_ptr,
                            inner_,
                        );
                    }
                    e => todo!("{e:?}"),
                };

                // If we reach the last element, we exit
                loop_
                    .local_get(i)
                    .local_get(len)
                    .i32_const(1)
                    .binop(BinaryOp::I32Sub)
                    .binop(BinaryOp::I32Eq)
                    .br_if(inner_block_id);

                // Else, increment i and continue the loop
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

        // Wipe the data so we can write the length of the vector
        outer_block
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(0)
            .i32_const(32)
            .memory_fill(compilation_ctx.memory_id);

        // Write the length in the slot data
        outer_block
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
    });
}

/// Adds the instructions to encode and save a vector (as a field in a struct) into storage.
pub fn add_read_and_decode_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    data_ptr: LocalId,        // where to write the decoded data
    slot_ptr: LocalId,        // pointer to the slot of the vector header
    inner: &IntermediateType, // vector inner type
) {
    #[cfg(feature = "inject-host-debug-fns")]
    let (print_i32, print_i64, print_memory_from, print_address, print_separator, print_u128) = {
        crate::inject_debug_fns(module);
        crate::declare_host_debug_functions!(module)
    };

    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Locals
    let len = module.locals.add(ValType::I32);
    let elem_slot_ptr = module.locals.add(ValType::I32);

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
        .local_tee(len);

    // Check if length == 0
    builder.i32_const(0).binop(BinaryOp::I32Eq).if_else(
        None,
        |then| {
            // If the vector is empty (len == 0) we allocate 32 bytes of empty data and skip the rest of the instructions.
            // This is neeeded because the caller (add_read_and_decode_storage_struct_instructions) is going to save the field pointer in the struct data.
            // If we dont reserve this memory, we can end up reading garbage data and messing up the decoding.
            then.i32_const(32)
                .call(compilation_ctx.allocator)
                .local_set(data_ptr);
        },
        |else_| {
            // Stack size of the inner type
            let stack_size = inner.stack_data_size() as i32;

            // Element size in STORAGE
            // TODO: look out for structs, they have field size = 0.
            let elem_size = field_size(inner, compilation_ctx);

            // Allocate memory for the vector and write the header data
            IVector::allocate_vector_with_header(
                else_,
                compilation_ctx,
                data_ptr,
                len,
                len,
                stack_size,
            );

            // Calculate the slot of the first element
            else_
                .local_get(slot_ptr)
                .i32_const(32)
                .local_get(elem_slot_ptr)
                .call(native_keccak);

            // Load the first slot data
            else_
                .local_get(elem_slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_load);

            // Read the elements from the vector
            else_.block(None, |block| {
                let block_id = block.id();

                let i = module.locals.add(ValType::I32);
                let read_bytes_in_slot = module.locals.add(ValType::I32);

                // Set the aux locals to 0 to start the loop
                block.i32_const(0).local_set(i);
                block.i32_const(0).local_set(read_bytes_in_slot);
                block.loop_(None, |loop_| {
                    let loop_id = loop_.id();

                    loop_
                        .local_get(read_bytes_in_slot)
                        .i32_const(elem_size as i32)
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

                                then.i32_const(elem_size as i32)
                                    .local_set(read_bytes_in_slot);
                            },
                            |else_| {
                                else_
                                    .local_get(read_bytes_in_slot)
                                    .i32_const(elem_size as i32)
                                    .binop(BinaryOp::I32Add)
                                    .local_set(read_bytes_in_slot);
                            },
                        );

                    match inner {
                        IntermediateType::IBool
                        | IntermediateType::IU8
                        | IntermediateType::IU16
                        | IntermediateType::IU32
                        | IntermediateType::IU64 => {
                            // Determine store kind and swap function
                            let (store_kind, swap_fn) = if stack_size == 8 {
                                let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None);
                                (StoreKind::I64 { atomic: false }, swap_fn)
                            } else {
                                let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
                                (StoreKind::I32 { atomic: false }, swap_fn)
                            };

                            // Determine load kind
                            let load_kind = match elem_size {
                                1 => LoadKind::I32_8 {
                                    kind: ExtendedLoad::ZeroExtend,
                                },
                                2 => LoadKind::I32_16 {
                                    kind: ExtendedLoad::ZeroExtend,
                                },
                                4 => LoadKind::I32 { atomic: false },
                                8 => LoadKind::I64 { atomic: false },
                                _ => panic!("invalid element size {elem_size} for type {inner:?}"),
                            };

                            let tmp_data_ptr = module.locals.add(ValType::I32);
                            // Destination address of the element in memory
                            loop_.vec_elem_ptr(data_ptr, i, stack_size);
                            loop_.local_tee(tmp_data_ptr);

                            // Load the (u8, u16, u32, u64) value from slot data (plus offset)
                            loop_
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
                            if elem_size == 1 {
                                loop_.i32_const(24).binop(BinaryOp::I32ShrU);
                            } else if elem_size == 2 {
                                loop_.i32_const(16).binop(BinaryOp::I32ShrU);
                            }

                            // Save the value into the data memory
                            loop_.store(
                                compilation_ctx.memory_id,
                                store_kind,
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            );
                        }
                        IntermediateType::IU128 => {
                            let swap_fn =
                                RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));
                            let heap_elem_ptr = module.locals.add(ValType::I32);

                            // Allocate 16 bytes for the u128 element
                            loop_
                                .i32_const(IU128::HEAP_SIZE)
                                .call(compilation_ctx.allocator)
                                .local_tee(heap_elem_ptr);

                            // Source address (plus offset)
                            loop_
                                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                                .i32_const(32)
                                .binop(BinaryOp::I32Add)
                                .local_get(read_bytes_in_slot)
                                .binop(BinaryOp::I32Sub);

                            // Number of bytes to copy
                            loop_.i32_const(IU128::HEAP_SIZE);

                            // Copy the chunk of memory
                            loop_.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                            // Transform it to LE
                            loop_
                                .local_get(heap_elem_ptr)
                                .local_get(heap_elem_ptr)
                                .call(swap_fn);

                            // Store the pointer to the copied u128 into the data memory
                            loop_
                                .vec_elem_ptr(data_ptr, i, stack_size)
                                .local_get(heap_elem_ptr)
                                .store(
                                    compilation_ctx.memory_id,
                                    StoreKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
                        }
                        IntermediateType::IU256 => {
                            let swap_fn =
                                RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));
                            let heap_elem_ptr = module.locals.add(ValType::I32);

                            // Allocate 32 bytes for the u256 element
                            loop_
                                .i32_const(IU256::HEAP_SIZE)
                                .call(compilation_ctx.allocator)
                                .local_tee(heap_elem_ptr);

                            // Source address
                            loop_.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                            // Number of bytes to copy
                            loop_.i32_const(IU256::HEAP_SIZE);

                            // Copy the chunk of memory
                            loop_.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                            // Transform it to LE
                            loop_
                                .local_get(heap_elem_ptr)
                                .local_get(heap_elem_ptr)
                                .call(swap_fn);

                            // Store the pointer to the copied u256 into the data memory
                            loop_
                                .vec_elem_ptr(data_ptr, i, stack_size)
                                .local_get(heap_elem_ptr)
                                .store(
                                    compilation_ctx.memory_id,
                                    StoreKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
                        }
                        IntermediateType::IAddress | IntermediateType::ISigner => {
                            let heap_elem_ptr = module.locals.add(ValType::I32);
                            // Allocate memory for the address
                            // TODO: for this to work we are saving 32 and using only 20. Debug this.
                            loop_
                                .i32_const(32)
                                .call(compilation_ctx.allocator)
                                .local_tee(heap_elem_ptr)
                                .i32_const(12)
                                .binop(BinaryOp::I32Add);

                            // Source address
                            // The offset is fixed because only one element address fits in a slot.
                            loop_
                                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                                .i32_const(12)
                                .binop(BinaryOp::I32Add);

                            // Number of bytes to copy
                            loop_.i32_const(20);

                            // Copy the chunk of memory
                            loop_.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                            // Store the pointer to the copied address into the data memory
                            loop_
                                .vec_elem_ptr(data_ptr, i, stack_size)
                                .local_get(heap_elem_ptr)
                                .store(
                                    compilation_ctx.memory_id,
                                    StoreKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
                        }
                        // IntermediateType::IStruct { module_id, index } => {
                        //     let child_struct = compilation_ctx
                        //         .get_user_data_type_by_index(module_id, *index)
                        //         .unwrap();

                        //     // Read the child struct
                        //     let (child_struct_ptr, read_bytes) =
                        //         add_read_and_decode_storage_struct_instructions(
                        //             module,
                        //             builder,
                        //             compilation_ctx,
                        //             slot_ptr,
                        //             child_struct,
                        //             true,
                        //             read_bytes_in_slot,
                        //         );

                        //     read_bytes_in_slot = read_bytes;

                        //     builder.local_get(child_struct_ptr).local_set(field_ptr);
                        // }
                        // IntermediateType::IGenericStructInstance {
                        //     module_id,
                        //     index,
                        //     types,
                        // } => {
                        //     let child_struct = compilation_ctx
                        //         .get_user_data_type_by_index(module_id, *index)
                        //         .unwrap();
                        //     let child_struct = child_struct.instantiate(types);

                        //     // Read the child struct
                        //     let (child_struct_ptr, read_bytes) =
                        //         add_read_and_decode_storage_struct_instructions(
                        //             module,
                        //             builder,
                        //             compilation_ctx,
                        //             slot_ptr,
                        //             &child_struct,
                        //             true,
                        //             read_bytes_in_slot,
                        //         );

                        //     read_bytes_in_slot = read_bytes;

                        //     builder.local_get(child_struct_ptr).local_set(field_ptr);
                        // }
                        // IntermediateType::IExternalUserData {
                        //     module_id,
                        //     identifier,
                        //     ..
                        // } if Uid::is_vm_type(module_id, identifier) => {
                        //     // Here we need to reconstruct the UID struct. To do that we first allocate 4 bytes
                        //     // that will contain the pointer to the UID struct data
                        //     //
                        //     // After that we need to create the ID struct. So we allocate 4 bytes for the first
                        //     // field's pointer, and 32 bytes that will hold the actual data.

                        //     // Create a pointer for the value. This pointer will point to the struct ID
                        //     builder
                        //         .i32_const(4)
                        //         .call(compilation_ctx.allocator)
                        //         .local_set(field_ptr);

                        //     let id_struct_ptr = module.locals.add(ValType::I32);
                        //     let id_field_ptr = module.locals.add(ValType::I32);

                        //     // Recreate the ID struct

                        //     // First, 4 bytes for the pointer that points to the ID
                        //     builder
                        //         .i32_const(4)
                        //         .call(compilation_ctx.allocator)
                        //         .local_set(id_struct_ptr);

                        //     // 32 bytes to save the actual id
                        //     builder
                        //         .i32_const(32)
                        //         .call(compilation_ctx.allocator)
                        //         .local_tee(id_field_ptr);

                        //     // Source address (plus offset)
                        //     builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                        //     // Number of bytes to copy
                        //     builder.i32_const(32);

                        //     // Copy the chunk of memory
                        //     builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                        //     // Point the id_field_ptr to the data
                        //     builder
                        //         .local_get(id_struct_ptr)
                        //         .local_get(id_field_ptr)
                        //         .store(
                        //             compilation_ctx.memory_id,
                        //             StoreKind::I32 { atomic: false },
                        //             MemArg {
                        //                 align: 0,
                        //                 offset: 0,
                        //             },
                        //         );

                        //     // Write the field_ptr with the address of the ID struct
                        //     builder.local_get(field_ptr).local_get(id_struct_ptr).store(
                        //         compilation_ctx.memory_id,
                        //         StoreKind::I32 { atomic: false },
                        //         MemArg {
                        //             align: 0,
                        //             offset: 0,
                        //         },
                        //     );
                        // }
                        // IntermediateType::IExternalUserData {
                        //     module_id,
                        //     identifier,
                        //     types,
                        // } => {
                        //     let external_data = compilation_ctx
                        //         .get_external_module_data(module_id, identifier, types)
                        //         .unwrap();

                        //     match external_data {
                        //         ExternalModuleData::Struct(child_struct) => {
                        //             // Read the child struct
                        //             let (child_struct_ptr, read_bytes) =
                        //                 add_read_and_decode_storage_struct_instructions(
                        //                     module,
                        //                     builder,
                        //                     compilation_ctx,
                        //                     slot_ptr,
                        //                     &child_struct,
                        //                     true,
                        //                     read_bytes_in_slot,
                        //                 );

                        //             read_bytes_in_slot = read_bytes;

                        //             builder.local_get(child_struct_ptr).local_set(field_ptr);
                        //         }
                        //         ExternalModuleData::Enum(_) => {
                        //             todo!();
                        //         }
                        //     }
                        // }
                        IntermediateType::IVector(inner_) => {
                            let inner_data_ptr = module.locals.add(ValType::I32);
                            let inner_slot_ptr = module.locals.add(ValType::I32);

                            // Duplicate the element to avoid overwriting it
                            loop_.local_get(elem_slot_ptr).local_set(inner_slot_ptr);

                            add_read_and_decode_storage_vector_instructions(
                                module,
                                loop_,
                                compilation_ctx,
                                inner_data_ptr,
                                inner_slot_ptr,
                                inner_,
                            );

                            // Save the inner vector ptr in the parent vector at position i
                            loop_
                                .vec_elem_ptr(data_ptr, i, stack_size)
                                .local_get(inner_data_ptr)
                                .store(
                                    compilation_ctx.memory_id,
                                    StoreKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
                        }
                        _ => todo!(),
                    };

                    // If we reach the last element, we exit
                    loop_
                        .local_get(i)
                        .local_get(len)
                        .i32_const(1)
                        .binop(BinaryOp::I32Sub)
                        .binop(BinaryOp::I32Eq)
                        .br_if(block_id);

                    // Else, increment i and continue the loop
                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_set(i)
                        .br(loop_id);
                });
            });
        },
    );
}

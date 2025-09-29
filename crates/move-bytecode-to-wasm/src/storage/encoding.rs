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
    data::{
        DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET,
        DATA_STORAGE_OBJECT_OWNER_OFFSET,
    },
    hostio::host_functions::{native_keccak256, storage_cache_bytes32, storage_load_bytes32},
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        IntermediateType,
        heap_integers::{IU128, IU256},
        structs::IStruct,
        vector::IVector,
    },
    vm_handled_types::{VmHandledType, named_id::NamedId, uid::Uid},
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
pub fn add_encode_and_save_into_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    struct_ptr: LocalId,
    slot_ptr: LocalId,
    struct_: &IStruct,
    written_bytes_in_slot: LocalId,
) {
    let (storage_cache, _) = storage_cache_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    for (index, field) in struct_.fields.iter().enumerate() {
        if matches!(
            field,
            IntermediateType::IStruct { module_id, index, ..}
                | IntermediateType::IGenericStructInstance { module_id, index, ..}
                    if Uid::is_vm_type(module_id, *index, compilation_ctx)
                        || NamedId::is_vm_type(module_id, *index, compilation_ctx))
        {
            if struct_.fields.len() == 1 {
                let get_struct_id_fn =
                    RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
                // This is a unique scenario where the struct contains only a UID field.
                // The problem arises because UIDs are not stored in the storage, resulting in no data being saved for this struct.
                // Consequently, when the `locate_storage_data` function is called to retrieve the struct from storage, it emits a trap due to the slot being empty.
                // To prevent this issue, we explicitly write the UID into the designated storage slot.
                let child_struct_id_ptr = module.locals.add(ValType::I32);

                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(child_struct_id_ptr);

                builder
                    .local_get(struct_ptr)
                    .call(get_struct_id_fn)
                    .local_set(child_struct_id_ptr);

                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .local_get(child_struct_id_ptr)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            }
        } else {
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
                        // Save previous slot (maybe not needed...)
                        then.local_get(slot_ptr)
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .call(storage_cache);

                        // Wipe the data so we can fill it with new data
                        then.i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .i32_const(0)
                            .i32_const(32)
                            .memory_fill(compilation_ctx.memory_id);

                        then.local_get(slot_ptr)
                            .call(next_slot_fn)
                            .local_set(slot_ptr);

                        then.i32_const(field_size as i32)
                            .local_set(written_bytes_in_slot);
                    },
                    |else_| {
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

            // Encode the field and write it to DATA_SLOT_DATA_PTR_OFFSET
            add_encode_intermediate_type_instructions(
                module,
                builder,
                compilation_ctx,
                slot_ptr,
                struct_ptr,
                field,
                written_bytes_in_slot,
                true,
            );
        }
    }

    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);
}

/// Adds the instructions to read, decode from storage and build in memory a structure.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `slot_ptr` - storage's slot where the data will be saved
/// `struct_` - structural information of the struct to be encoded and saved
/// `reading_nested_struct` - if true, this function is called to read a nested struct inside
/// `read_bytes_in_slot` - number of bytes already read in the slot.
/// another struct.
///
/// # Returns
/// pointer where the read struct is allocated
pub fn add_read_and_decode_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    uid_ptr: LocalId,
    struct_: &IStruct,
    reading_nested_struct: bool,
    read_bytes_in_slot: LocalId,
) -> LocalId {
    let (storage_load, _) = storage_load_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Locals
    let struct_ptr = module.locals.add(ValType::I32);
    let field_ptr = module.locals.add(ValType::I32);

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

    for (index, field) in struct_.fields.iter().enumerate() {
        if matches!(
            field,
            IntermediateType::IStruct { module_id, index, ..}
                | IntermediateType::IGenericStructInstance { module_id, index, ..}
                    if Uid::is_vm_type(module_id, *index, compilation_ctx)
                        || NamedId::is_vm_type(module_id, *index, compilation_ctx))
        {
            // Save the struct pointer in the reserved space of the UID
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

            let uid_ptr_wrapper = module.locals.add(ValType::I32);

            // First, 4 bytes for the pointer that points to the ID
            builder
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_set(field_ptr);

            builder
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_set(uid_ptr_wrapper);

            // Point the id_field_ptr to the data
            builder.local_get(uid_ptr_wrapper).local_get(uid_ptr).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Write the data_ptr with the address of the ID struct
            builder
                .local_get(field_ptr)
                .local_get(uid_ptr_wrapper)
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
                .local_get(read_bytes_in_slot)
                .i32_const(field_size)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32GtS)
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

            add_decode_intermediate_type_instructions(
                module,
                builder,
                compilation_ctx,
                field_ptr,
                slot_ptr,
                struct_ptr,
                uid_ptr,
                field,
                read_bytes_in_slot,
            );
        }

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

    struct_ptr
}

/// Adds the instructions to encode and save a vector (as a field in a struct) into storage.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `vector_ptr` - pointer to the vector in memory
/// `slot_ptr` - pointer to the vector header slot
/// `inner` - inner type of the vector
pub fn add_encode_and_save_into_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    vector_ptr: LocalId,
    slot_ptr: LocalId,
    parent_struct_ptr: LocalId,
    inner: &IntermediateType,
) {
    // Host functions
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Locals
    let elem_slot_ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

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
        let stack_size = inner.stack_data_size() as i32;

        // Element size in storage
        let elem_size = field_size(inner, compilation_ctx) as i32;

        // First slot = keccak(header_slot)
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
                            then.i32_const(elem_size).local_set(written_bytes_in_slot);
                        },
                        |else_| {
                            // Increment the written bytes in slot by the element size
                            else_
                                .local_get(written_bytes_in_slot)
                                .i32_const(elem_size)
                                .binop(BinaryOp::I32Add)
                                .local_set(written_bytes_in_slot);
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

                // Encode the intermediate type and write it to DATA_SLOT_DATA_PTR_OFFSET
                add_encode_intermediate_type_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_slot_ptr,
                    parent_struct_ptr,
                    inner,
                    written_bytes_in_slot,
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
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `data_ptr` - pointer to the memory region where the vector data will be written
/// `slot_ptr` - pointer to the vector header slot
/// `inner` - inner type of the vector
pub fn add_read_and_decode_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    data_ptr: LocalId,
    slot_ptr: LocalId,
    parent_struct_ptr: LocalId,
    parent_uid_ptr: LocalId,
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
            // If the vector is empty (len == 0) we reserve 8 bytes of empty data and skip the rest of the instructions.
            // This is neeeded because the caller (add_read_and_decode_storage_struct_instructions) is going to save the field pointer in the struct data.
            // If we dont reserve this memory, we can end up reading garbage data and messing up the decoding.
            then.i32_const(8)
                .call(compilation_ctx.allocator)
                .local_set(data_ptr);
        },
        |else_| {
            // Stack size of the inner type
            let stack_size = inner.stack_data_size() as i32;

            // Element size in STORAGE
            let elem_size = field_size(inner, compilation_ctx) as i32;

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
                        parent_struct_ptr,
                        parent_uid_ptr,
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

/// Adds the instructions to encode and write an intermediate type to the storage slot.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - insturctions sequence builder
/// `compilation_ctx` - compilation context
/// `slot_ptr` - storage's slot where the data will be saved
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
    parent_struct_ptr: LocalId,
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
        IntermediateType::IStruct {
            module_id, index, ..
        }
        | IntermediateType::IGenericStructInstance {
            module_id, index, ..
        } => {
            // This section handles encoding of nested structs within parent structs.
            // The behavior differs based on whether the child struct has the 'key' ability:
            // - If child has 'key': stored as separate object under the parent key
            // - If child has no 'key': flattened into parent struct's data

            let child_struct_ptr = module.locals.add(ValType::I32);
            builder.local_set(child_struct_ptr);

            // Get child struct by (module_id, index)
            let child_struct = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .expect("struct not found");

            // If it's a generic instance, instantiate; otherwise use as-is
            let child_struct = if let IntermediateType::IGenericStructInstance { types, .. } = itype
            {
                child_struct.instantiate(types)
            } else {
                child_struct.clone()
            };

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

                // Calculate the slot for the child struct according to Solidity storage layout:
                // child_struct_slot = keccak256(child_struct_id || keccak256(parent_struct_id || 0))

                // Extract parent struct ID for slot calculation
                let parent_struct_id_ptr = module.locals.add(ValType::I32);
                builder
                    .local_get(parent_struct_ptr)
                    .call(get_struct_id_fn)
                    .local_set(parent_struct_id_ptr);

                // Extract child struct ID for slot calculation
                let child_struct_id_ptr = module.locals.add(ValType::I32);
                builder
                    .local_get(child_struct_ptr)
                    .call(get_struct_id_fn)
                    .local_set(child_struct_id_ptr);

                // Calculate the unique slot for the child struct
                builder
                    .local_get(parent_struct_id_ptr)
                    .local_get(child_struct_id_ptr)
                    .call(write_object_slot_fn);

                // Allocate memory for the child struct slot and copy the calculated
                // slot data to avoid overwriting during recursive encoding.

                // Allocate memory for child struct slot (32 bytes for slot data)
                let child_struct_slot_ptr = module.locals.add(ValType::I32);
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(child_struct_slot_ptr);

                // Copy the calculated slot data to the allocated memory
                builder
                    .local_get(child_struct_slot_ptr)
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
                    &child_struct,
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
                    &child_struct,
                    written_bytes_in_slot,
                );
            }
        }
        IntermediateType::IVector(inner) => {
            builder.local_set(val_32);

            add_encode_and_save_into_storage_vector_instructions(
                module,
                builder,
                compilation_ctx,
                val_32,
                slot_ptr,
                parent_struct_ptr,
                inner,
            );

            // TODO: is this needed?
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
/// `itype` - intermediate type to be decoded
/// `read_bytes_in_slot` - number of bytes already read in the slot.
#[allow(clippy::too_many_arguments)]
pub fn add_decode_intermediate_type_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    data_ptr: LocalId,
    slot_ptr: LocalId,
    parent_struct_ptr: LocalId,
    parent_uid_ptr: LocalId,
    itype: &IntermediateType,
    read_bytes_in_slot: LocalId,
) {
    // Stack and storage size of the type
    let stack_size = itype.stack_data_size() as i32;
    let storage_size = field_size(itype, compilation_ctx) as i32;

    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);

    // Runtime functions
    let get_struct_id_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
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
            let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

            // Allocate 16 bytes for the u128 element
            builder
                .i32_const(IU128::HEAP_SIZE)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr);

            // Source address (plus offset)
            builder
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .local_get(read_bytes_in_slot)
                .binop(BinaryOp::I32Sub);

            // Number of bytes to copy
            builder.i32_const(IU128::HEAP_SIZE);

            // Copy the chunk of memory
            builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

            // Transform it to LE
            builder
                .local_get(data_ptr)
                .local_get(data_ptr)
                .call(swap_fn);
        }
        IntermediateType::IU256 => {
            let swap_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

            // Allocate 32 bytes for the u256 element
            builder
                .i32_const(IU256::HEAP_SIZE)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr);

            // Source address
            builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

            // Number of bytes to copy
            builder.i32_const(IU256::HEAP_SIZE);

            // Copy the chunk of memory
            builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

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
        IntermediateType::IStruct {
            module_id, index, ..
        }
        | IntermediateType::IGenericStructInstance {
            module_id, index, ..
        } => {
            // ========================================================================
            // Handle Nested Struct Decoding
            // ========================================================================
            // This section handles decoding of nested structs from storage.
            // The behavior differs based on whether the child struct has the 'key' ability:
            // - If child has 'key': read UID from parent, calculate child slot, decode child
            // - If child has no 'key': decode child directly from current slot (flattened)

            // Get base definition by (module_id, index)
            let base_def = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .expect("struct not found");

            // If it's a generic instance, instantiate; otherwise use as-is
            let child_struct = if let IntermediateType::IGenericStructInstance { types, .. } = itype
            {
                base_def.instantiate(types)
            } else {
                base_def.clone()
            };

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

                // Get parent struct ID
                let parent_struct_id_ptr = module.locals.add(ValType::I32);
                builder
                    .local_get(parent_struct_ptr)
                    .call(get_struct_id_fn)
                    .local_set(parent_struct_id_ptr);

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
                // child_struct_slot = keccak256(child_struct_id || keccak256(parent_struct_id || 0))
                builder
                    .local_get(parent_struct_id_ptr)
                    .local_get(child_struct_id_ptr)
                    .call(write_object_slot_fn);

                // Allocate memory for the child struct slot and copy the calculated
                // slot data to avoid overwriting during recursive decoding.

                // Allocate memory for child struct slot (32 bytes for slot data)
                let child_struct_slot_ptr = module.locals.add(ValType::I32);
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(child_struct_slot_ptr);

                // Copy the calculated slot data to the allocated memory
                builder
                    .local_get(child_struct_slot_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Set the owner of the child struct to the parent struct id
                builder
                    .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
                    .local_get(parent_struct_id_ptr)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Reset read bytes counter for the child struct decoding
                builder.i32_const(0).local_set(read_bytes_in_slot);

                // Recursively decode the child struct from its dedicated slot
                // This will handle all fields of the child struct and reconstruct
                // the complete child struct object.
                let child_struct_ptr = add_read_and_decode_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    child_struct_slot_ptr,
                    child_struct_id_ptr,
                    &child_struct,
                    false,
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
                    parent_uid_ptr,
                    &child_struct,
                    true,
                    read_bytes_in_slot,
                );

                // Set the decoded child struct as the result
                builder.local_get(child_struct_ptr).local_set(data_ptr);
            }
        }
        IntermediateType::IVector(inner_) => {
            add_read_and_decode_storage_vector_instructions(
                module,
                builder,
                compilation_ctx,
                data_ptr,
                slot_ptr,
                parent_struct_ptr,
                parent_uid_ptr,
                inner_,
            );
        }
        _ => todo!(),
    };
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

        IntermediateType::IStruct {
            module_id, index, ..
        } if Uid::is_vm_type(module_id, *index, compilation_ctx) => 32,

        IntermediateType::IGenericStructInstance {
            module_id, index, ..
        } if NamedId::is_vm_type(module_id, *index, compilation_ctx) => 32,

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

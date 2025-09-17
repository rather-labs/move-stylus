use super::NativeFunction;
use crate::{
    CompilationContext,
    hostio::host_functions::{
        block_number, block_timestamp, emit_log, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    translation::intermediate_types::{address::IAddress, IntermediateType, structs::IStruct},
    utils::keccak_string_to_memory,
    vm_handled_types::{VmHandledType, uid::Uid},
    storage::encoding::field_size,
    data::DATA_SLOT_DATA_PTR_OFFSET,
    runtime::RuntimeFunction,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType, InstrSeqBuilder, LocalId,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn add_native_fresh_id_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let (native_keccak, _) = native_keccak256(module);
    let (block_number, _) = block_number(module);
    let (block_timestamp, _) = block_timestamp(module);
    let (storage_load_fn, _) = storage_load_bytes32(module);
    let (storage_cache_fn, _) = storage_cache_bytes32(module);
    let (storage_flush_cache_fn, _) = storage_flush_cache(module);
    let (emit_log_fn, _) = emit_log(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let id_ptr = module.locals.add(ValType::I32);
    let data_to_hash_ptr = module.locals.add(ValType::I32);
    let counter_key_ptr = module.locals.add(ValType::I32); // Pointer for the storage key
    let counter_value_ptr = module.locals.add(ValType::I32); // Pointer to receive value read from storage

    let mut builder = function
        .name(NativeFunction::NATIVE_FRESH_ID.to_owned())
        .func_body();

    // Counter key
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(counter_key_ptr);

    // Counter value
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(counter_value_ptr);

    // ID
    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_set(id_ptr);

    // Data to hash: block timestamp (8 bytes) + block number (8 bytes) + counter (4 bytes)
    builder
        .i32_const(20)
        .call(compilation_ctx.allocator)
        .local_set(data_to_hash_ptr);

    // Store the keccak256 hash of the counter key into linear memory at #counter_key_ptr
    keccak_string_to_memory(
        &mut builder,
        compilation_ctx,
        "global_counter_key",
        counter_key_ptr,
    );

    // Load the counter from storage
    builder
        .local_get(counter_key_ptr)
        .local_get(counter_value_ptr)
        .call(storage_load_fn);

    // Increment the counter and store it in the local variable
    builder
        .local_get(counter_value_ptr)
        .local_get(counter_value_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(u32::MAX as i32)
        .binop(BinaryOp::I32LtU)
        .if_else(
            Some(ValType::I32),
            |then| {
                then.local_get(counter_value_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .i32_const(1)
                    .binop(BinaryOp::I32Add);
            },
            |else_| {
                else_.i32_const(0);
            },
        );

    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // - Store the block timestamp in the first 8 bytes at #data_to_hash
    // - Store the block number in the next 8 bytes
    // - Store the counter in the final 32 bytes
    builder
        .local_get(data_to_hash_ptr)
        .call(block_timestamp)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_get(data_to_hash_ptr)
        .call(block_number)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 8,
            },
        )
        .local_get(data_to_hash_ptr)
        .local_get(counter_value_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 16,
            },
        );

    // Hash the data to generate the ID
    builder
        .local_get(data_to_hash_ptr)
        .i32_const(20)
        .local_get(id_ptr)
        .call(native_keccak);

    // Update storage, flushing the cache
    builder
        .local_get(counter_key_ptr)
        .local_get(counter_value_ptr)
        .call(storage_cache_fn)
        .i32_const(1)
        .call(storage_flush_cache_fn);

    // Emit log with the ID
    builder
        .local_get(id_ptr)
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_fn);

    // Return the ID ptr
    builder.local_get(id_ptr);

    function.finish(vec![], &mut module.funcs)
}

/// This function adds instructions to recursively delete all storage slots
/// associated with a struct, including its fields and any nested structures or vectors.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - instructions sequence builder
/// `compilation_ctx` - compilation context containing type information
/// `slot_ptr` - pointer to the storage slot where the struct is stored
/// `struct_` - structural information of the struct to be deleted
/// `used_bytes_in_slot` - number of bytes already used in the current slot
pub fn add_delete_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    struct_: &IStruct,
    used_bytes_in_slot: LocalId,
) {
    let (storage_cache, _) = storage_cache_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Iterate over the fields of the struct and delete them
    for field in struct_.fields.iter() {
        let field_size = field_size(field, compilation_ctx) as i32;
        builder
            // Check if used_bytes_in_slot + field_size > 32
            .local_get(used_bytes_in_slot)
            .i32_const(field_size)
            .binop(BinaryOp::I32Add)
            .i32_const(32)
            .binop(BinaryOp::I32GtS)
            .if_else(
                None,
                |then| {
                    // Wipe the slot data
                    then.local_get(slot_ptr)
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .call(storage_cache);

                    // Set slot_ptr to the next slot
                    then.local_get(slot_ptr)
                        .call(next_slot_fn)
                        .local_set(slot_ptr);

                    // Set used_bytes_in_slot to field_size
                    then.i32_const(field_size).local_set(used_bytes_in_slot);
                },
                |else_| {
                    // Increment used_bytes_in_slot by field_size
                    else_
                        .local_get(used_bytes_in_slot)
                        .i32_const(field_size)
                        .binop(BinaryOp::I32Add)
                        .local_set(used_bytes_in_slot);
                },
            );

        match field {
            IntermediateType::IStruct { module_id, index }
                if !Uid::is_vm_type(module_id, *index, compilation_ctx) =>
            {
                let child_struct = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();

                // Delete the child struct
                // If the child struct has key, then its stored under the parent object key in storage.
                // We need to calculate its slot and pass that to add_delete_storage_struct_instructions
                let has_key = false;
                if has_key {
                    // TODO: Implement this
                    // call write_object_slot with [parent_struct_id_ptr, child_struct_id_ptr]
                    // use that slot_ptr in add_delete_storage_struct_instructions
                } else {
                    // If the struct does not have key, then we can delete it directly
                    add_delete_storage_struct_instructions(
                        module,
                        builder,
                        compilation_ctx,
                        slot_ptr,
                        child_struct,
                        used_bytes_in_slot,
                    );
                }
            }
            IntermediateType::IVector(inner) => {
                // If the field is a vector, add the corresponding instructions to delete it
                add_delete_storage_vector_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    slot_ptr,
                    inner,
                );
            }
            _ => {}
        }
    }

    // Wipe out the last slot before exiting
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);
}

/// This function adds instructions to recursively delete all storage slots
/// associated with a vector, including its header slot and all element slots. It handles
/// vectors of any type including primitive types, structs, and nested vectors.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - instructions sequence builder
/// `compilation_ctx` - compilation context containing type information
/// `slot_ptr` - pointer to the storage slot where the vector header is stored
/// `inner` - type information of the vector elements
pub fn add_delete_storage_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    inner: &IntermediateType,
) {
    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    // Locals
    let len = module.locals.add(ValType::I32);
    let elem_slot_ptr = module.locals.add(ValType::I32);

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
        .local_set(len);

    // Wipe the slot data memory again
    // This is important because we are going to use it to wipe the vector slots
    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(0)
        .i32_const(32)
        .memory_fill(compilation_ctx.memory_id);

    // Wipe the header slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);

    builder.block(None, |block| {
        let block_id = block.id();

        // Check if length == 0. If so, skip the rest of the instructions.
        block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(block_id);

        // Element size in STORAGE
        let elem_size = field_size(inner, compilation_ctx) as i32;

        // Calculate the slot of the first element: keccak(header)
        block
            .local_get(slot_ptr)
            .i32_const(32)
            .local_get(elem_slot_ptr)
            .call(native_keccak);

        // Set the aux locals to 0 to start the loop
        let i = module.locals.add(ValType::I32);
        let used_bytes_in_slot = module.locals.add(ValType::I32);
        block.i32_const(0).local_set(i);
        block.i32_const(0).local_set(used_bytes_in_slot);
        block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            loop_
                .local_get(used_bytes_in_slot)
                .i32_const(elem_size)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32GtS)
                .if_else(
                    None,
                    // If used_bytes_in_slot + elem_size > 32, wipe the slot and advance the elem_slot_ptr
                    |then| {
                        // Wipe the slot
                        then.local_get(elem_slot_ptr)
                            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                            .call(storage_cache);

                        // Calculate next slot
                        then.local_get(elem_slot_ptr)
                            .call(next_slot_fn)
                            .local_set(elem_slot_ptr);

                        // Set used_bytes_in_slot to elem_size
                        then.i32_const(elem_size).local_set(used_bytes_in_slot);
                    },
                    // If used_bytes_in_slot + elem_size <= 32, increment used_bytes_in_slot by elem_size
                    |else_| {
                        // Increment used_bytes_in_slot by elem_size
                        else_
                            .local_get(used_bytes_in_slot)
                            .i32_const(elem_size)
                            .binop(BinaryOp::I32Add)
                            .local_set(used_bytes_in_slot);
                    },
                );

            match inner {
                IntermediateType::IStruct { module_id, index }
                    if !Uid::is_vm_type(module_id, *index, compilation_ctx) =>
                {
                    let child_struct = compilation_ctx
                        .get_struct_by_index(module_id, *index)
                        .unwrap();

                    // Delete the child struct
                    // If the child struct has key, then its stored under the parent object key in storage.
                    // We need to calculate its slot and pass that to add_delete_storage_struct_instructions
                    let has_key = false;
                    if has_key {
                        // TODO: Implement this
                        // call write_object_slot with [parent_struct_id_ptr, child_struct_id_ptr]
                        // use that slot_ptr in add_delete_storage_struct_instructions
                    } else {
                        // If the struct does not have key, then we can delete it directly

                        // This function modifies the original elem_slot_ptr passed as argument
                        // After exiting the function, elem_slot_ptr is advanced and used_bytes_in_slot is updated
                        add_delete_storage_struct_instructions(
                            module,
                            loop_,
                            compilation_ctx,
                            elem_slot_ptr,
                            child_struct,
                            used_bytes_in_slot,
                        );
                    }
                }
                IntermediateType::IVector(inner_) => {
                    // Delete the vector recursively
                    // This function does not modify the original elem_slot_ptr passed as argument
                    // elem_slot_ptr is copied and used as the new header slot pointer
                    add_delete_storage_vector_instructions(
                        module,
                        loop_,
                        compilation_ctx,
                        elem_slot_ptr,
                        inner_,
                    );
                }
                _ => {}
            }

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

    // Delete the last slot before exiting
    builder
        .local_get(elem_slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_cache);
}

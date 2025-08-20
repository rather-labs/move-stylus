use super::NativeFunction;
use crate::{
    CompilationContext,
    data::{
        DATA_FROZEN_OBJECTS_KEY_OFFSET, DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
        DATA_SLOT_DATA_PTR_OFFSET,
    },
    hostio::host_functions::{
        block_number, block_timestamp, emit_log, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    runtime::RuntimeFunction,
    storage,
    translation::intermediate_types::address::IAddress,
    translation::intermediate_types::structs::IStruct,
    utils::keccak_string_to_memory,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
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

/// Generates a function that deletes an object from storage.
///
/// This function:
/// 1. Validates the object is not frozen (frozen objects cannot be deleted).
/// 2. Locates the storage slot of the object.
/// 3. Clears the storage slot and any additional slots occupied by the struct fields.
/// 4. Flushes the cache to finalize the deletion.
///
/// Arguments:
/// - struct_ptr
pub fn add_delete_object_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let name = format!("{}_{hash}", NativeFunction::NATIVE_DELETE_OBJECT);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));
    let locate_struct_slot_fn =
        RuntimeFunction::LocateStructSlot.get(module, Some(compilation_ctx));
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));

    let (emit_log_fn, _) = emit_log(module);
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (storage_flush_cache, _) = storage_flush_cache(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let slot_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);

    // Verify if the object is frozen; if not, continue.
    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Sub)
        .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
        .i32_const(32)
        .call(equality_fn);

    builder.if_else(
        None,
        |then| {
            // Emit an unreachable if the object is frozen
            then.unreachable();
        },
        |else_| {
            // Calculate the object slot in the storage (saved in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            else_
                .local_get(struct_ptr)
                .call(locate_struct_slot_fn)
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .local_set(slot_ptr);

            else_
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .i32_const(32)
                .i32_const(0)
                .call(emit_log_fn);

            // Wipe the slot data placeholder. We will use it to erase the slots in the storage
            else_
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(0)
                .i32_const(32)
                .memory_fill(compilation_ctx.memory_id);

            // Wipe out the first slot
            else_
                .local_get(slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_cache);

            // Loop through each field in the struct and clear the corresponding storage slots.
            let mut slot_used_bytes = 0;
            for field in struct_.fields.iter() {
                let field_size = storage::encoding::field_size(field, compilation_ctx);
                if slot_used_bytes + field_size > 32 {
                    else_
                        .local_get(slot_ptr)
                        .call(next_slot_fn)
                        .local_tee(slot_ptr)
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .call(storage_cache);

                    slot_used_bytes = field_size;
                } else {
                    slot_used_bytes += field_size;
                }
            }

            else_.i32_const(1).call(storage_flush_cache);
        },
    );

    function.finish(vec![struct_ptr], &mut module.funcs)
}

use super::NativeFunction;
use crate::{
    CompilationContext,
    hostio::host_functions::{
        block_number, block_timestamp, emit_log, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    translation::intermediate_types::address::IAddress,
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

    // use crate::declare_host_debug_functions;
    // let (print_i32, _, _, _, _, _) = declare_host_debug_functions!(module);

    // builder.i32_const(1).call(print_i32);

    // Counter key
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(counter_key_ptr);

    // builder.i32_const(2).call(print_i32);
    // Counter value
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(counter_value_ptr);

    // builder.i32_const(3).call(print_i32);
    // ID
    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_set(id_ptr);

    // builder.i32_const(4).call(print_i32);
    // Data to hash: block timestamp (8 bytes) + block number (8 bytes) + counter (4 bytes)
    builder
        .i32_const(20)
        .call(compilation_ctx.allocator)
        .local_set(data_to_hash_ptr);

    // builder.i32_const(5).call(print_i32);
    // Store the keccak256 hash of the counter key into linear memory at #counter_key_ptr
    keccak_string_to_memory(
        &mut builder,
        compilation_ctx,
        "global_counter_key",
        counter_key_ptr,
    );

    // builder.i32_const(6).call(print_i32);
    // Load the counter from storage
    builder
        .local_get(counter_key_ptr)
        .local_get(counter_value_ptr)
        .call(storage_load_fn);

    // builder.i32_const(7).call(print_i32);
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

    // builder.i32_const(8).call(print_i32);
    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // builder.i32_const(9).call(print_i32);
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

    // builder.i32_const(10).call(print_i32);
    // Hash the data to generate the ID
    builder
        .local_get(data_to_hash_ptr)
        .i32_const(20)
        .local_get(id_ptr)
        .call(native_keccak);

    // builder.i32_const(11).call(print_i32);
    // Update storage, flushing the cache
    builder
        .local_get(counter_key_ptr)
        .local_get(counter_value_ptr)
        .call(storage_cache_fn)
        .i32_const(1)
        .call(storage_flush_cache_fn);

    // builder.i32_const(12).call(print_i32);
    // Emit log with the ID
    builder
        .local_get(id_ptr)
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_fn);

    // builder.i32_const(13).call(print_i32);
    // Return the ID ptr
    builder.local_get(id_ptr);

    function.finish(vec![], &mut module.funcs)
}

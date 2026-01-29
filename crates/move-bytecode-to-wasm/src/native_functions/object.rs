// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use super::{NativeFunction, error::NativeFunctionError};
use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    hostio::host_functions::{
        block_number, block_timestamp, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    translation::intermediate_types::{IntermediateType, address::IAddress},
    utils::keccak_string_to_memory,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn add_compute_named_id_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_COMPUTE_NAMED_ID,
        compilation_ctx,
        &[itype],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    if let IntermediateType::IStruct {
        module_id, index, ..
    } = itype
    {
        let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

        let id_ptr = module.locals.add(ValType::I32);

        let mut builder = function.name(name).func_body();

        // ID
        builder
            .i32_const(IAddress::HEAP_SIZE)
            .call(compilation_ctx.allocator)
            .local_set(id_ptr);

        let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

        // Store the keccak256 hash of the counter key into linear memory at #counter_key_ptr
        keccak_string_to_memory(&mut builder, compilation_ctx, &struct_.identifier, id_ptr);

        // Return the ID ptr
        builder.local_get(id_ptr);

        Ok(function.finish(vec![], &mut module.funcs))
    } else {
        Err(NativeFunctionError::WrongGenericType(
            NativeFunction::NATIVE_COMPUTE_NAMED_ID.to_owned(),
            itype.clone(),
        ))
    }
}

/// Takes a reference of a NamedId<T> and returns it casted as a UID
///
/// This function is used internally by the stylus framework
pub fn add_as_uid_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let name = NativeFunction::get_function_name(NativeFunction::NATIVE_AS_UID, module_id);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let named_id_ptr = module.locals.add(ValType::I32);

    let mut builder = function.name(name).func_body();

    let result = module.locals.add(ValType::I32);
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(result)
        .local_get(named_id_ptr)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    builder.local_get(result);

    function.finish(vec![named_id_ptr], &mut module.funcs)
}

pub fn add_native_fresh_id_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let name = NativeFunction::get_function_name(NativeFunction::NATIVE_FRESH_ID, module_id);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let (native_keccak, _) = native_keccak256(module);
    let (block_number, _) = block_number(module);
    let (block_timestamp, _) = block_timestamp(module);
    let (storage_load_fn, _) = storage_load_bytes32(module);
    let (storage_cache_fn, _) = storage_cache_bytes32(module);
    let (storage_flush_cache_fn, _) = storage_flush_cache(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let id_ptr = module.locals.add(ValType::I32);
    let data_to_hash_ptr = module.locals.add(ValType::I32);
    let counter_key_ptr = module.locals.add(ValType::I32); // Pointer for the storage key
    let counter_value_ptr = module.locals.add(ValType::I32); // Pointer to receive value read from storage

    let mut builder = function.name(name).func_body();

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

    // Return the ID ptr
    builder.local_get(id_ptr);

    function.finish(vec![], &mut module.funcs)
}

use std::rc::Rc;

use super::{NativeFunction, error::NativeFunctionError};
use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    data::{DATA_SLOT_DATA_PTR_OFFSET, DATA_ZERO_OFFSET},
    hostio::host_functions::{
        block_number, block_timestamp, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    runtime::RuntimeFunction,
    storage::storage_layout::field_size,
    translation::intermediate_types::{IntermediateType, address::IAddress, structs::IStruct},
    utils::keccak_string_to_memory,
    vm_handled_types::{VmHandledType, is_uid_or_named_id, named_id::NamedId, uid::Uid},
};
use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
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

/// This function adds instructions to recursively delete all storage slots
/// associated with a struct, including its fields and any nested structures or vectors.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - instructions sequence builder
/// `compilation_ctx` - compilation context containing type information
/// `slot_ptr` - pointer to the storage slot where the struct is stored
/// `slot_offset` - number of bytes already used in the current slot
/// `struct_` - structural information of the struct to be deleted
pub fn add_delete_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    slot_offset: LocalId,
    struct_: &IStruct,
) -> Result<(), NativeFunctionError> {
    let (storage_cache, _) = storage_cache_bytes32(module);

    // Iterate over the fields of the struct and delete them
    for field in struct_.fields.iter() {
        if is_uid_or_named_id(field, compilation_ctx)? {
            // If the field is a UID or NamedId, do nothing as UIDs are not stored in storage
            continue;
        }
        let field_size = field_size(field, compilation_ctx)
            .map_err(|e| NativeFunctionError::Storage(Rc::new(e)))? as i32;

        add_delete_field_instructions(
            module,
            builder,
            compilation_ctx,
            slot_ptr,
            slot_offset,
            field,
            field_size,
        )?;
    }

    // Wipe out the last slot before exiting
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_ZERO_OFFSET)
        .call(storage_cache);

    Ok(())
}

pub fn add_delete_storage_enum_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    slot_offset: LocalId,
    itype: &IntermediateType,
) -> Result<(), NativeFunctionError> {
    let (storage_cache, _) = storage_cache_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx))?;
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx))?;
    let compute_enum_storage_tail_position_fn = RuntimeFunction::ComputeEnumStorageTailPosition
        .get_generic(module, compilation_ctx, &[itype])?;

    // Compute the end slot
    let tail_slot_ptr = module.locals.add(ValType::I32);

    builder
        .local_get(slot_ptr)
        .local_get(slot_offset)
        .call(compute_enum_storage_tail_position_fn)
        .local_set(tail_slot_ptr);

    builder.loop_(None, |loop_| {
        let loop_id = loop_.id();

        loop_
            .local_get(slot_ptr)
            .local_get(tail_slot_ptr)
            .i32_const(32)
            .call(equality_fn)
            .if_else(
                None,
                |_| {
                    // Fall-through: Exit the loop if the current slot is equal to the end slot
                },
                |else_| {
                    // Delete the current slot
                    else_
                        .local_get(slot_ptr)
                        .i32_const(DATA_ZERO_OFFSET)
                        .call(storage_cache);

                    // Advance to the next slot and continue
                    else_
                        .local_get(slot_ptr)
                        .call(next_slot_fn)
                        .local_set(slot_ptr)
                        .br(loop_id);
                },
            );
    });

    // Load the tail offset from the last 4 bytes of the data pointer
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
) -> Result<(), NativeFunctionError> {
    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);
    let (storage_cache, _) = storage_cache_bytes32(module);
    let (native_keccak, _) = native_keccak256(module);

    // Runtime functions
    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None)?;

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

    // Wipe the header slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_ZERO_OFFSET)
        .call(storage_cache);

    let mut inner_result = Ok(());
    builder.block(None, |block| {
        let block_id = block.id();

        // Check if length == 0. If so, skip the rest of the instructions.
        block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(block_id);

        // Element size in STORAGE
        match field_size(inner, compilation_ctx) {
            Ok(elem_size) => {
                // Calculate the slot of the first element: keccak(header)
                block
                    .local_get(slot_ptr)
                    .i32_const(32)
                    .local_get(elem_slot_ptr)
                    .call(native_keccak);

                // Set the aux locals to 0 to start the loop
                let i = module.locals.add(ValType::I32);
                let elem_slot_offset = module.locals.add(ValType::I32);
                block.i32_const(0).local_set(i);
                block.i32_const(0).local_set(elem_slot_offset);

                block.block(None, |inner_block| {
                    let inner_block_id = inner_block.id();
                    inner_block.loop_(None, |loop_| {
                        let loop_id = loop_.id();

                        inner_result = add_delete_field_instructions(
                            module,
                            loop_,
                            compilation_ctx,
                            elem_slot_ptr,
                            elem_slot_offset,
                            inner,
                            elem_size as i32,
                        );

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
                // Delete the last slot before exiting
                block
                    .local_get(elem_slot_ptr)
                    .i32_const(DATA_ZERO_OFFSET)
                    .call(storage_cache);
            }
            Err(e) => {
                inner_result = Err(NativeFunctionError::Storage(e.into()));
            }
        }
    });
    inner_result?;

    Ok(())
}

/// This function extracts common logic to wipe storage slots,
/// recursively calling the struct/vector delete functions depending on the type.
///
/// # Arguments
/// `module` - walrus module
/// `builder` - instructions sequence builder
/// `compilation_ctx` - compilation context containing type information
/// `slot_ptr` - pointer to the storage slot where the data is stored
/// `itype` - intermediate type of the element to be deleted
/// `size` - size of the itype in storage
/// `slot_offset` - number of bytes already used in the current slot
pub fn add_delete_field_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    slot_offset: LocalId,
    itype: &IntermediateType,
    size: i32,
) -> Result<(), NativeFunctionError> {
    let accumulate_or_advance_slot_delete_fn =
        RuntimeFunction::AccumulateOrAdvanceSlotDelete.get(module, Some(compilation_ctx))?;

    // Use accumulate_or_advance_slot with mode=2 (delete) to handle slot advancement
    // Mode 2 will wipe the slot to zero before advancing when needed
    builder
        .local_get(slot_ptr)
        .local_get(slot_offset)
        .i32_const(size)
        .call(accumulate_or_advance_slot_delete_fn)
        .local_set(slot_offset);

    match itype {
        IntermediateType::IStruct {
            module_id, index, ..
        }
        | IntermediateType::IGenericStructInstance {
            module_id, index, ..
        } if !Uid::is_vm_type(module_id, *index, compilation_ctx)?
            && !NamedId::is_vm_type(module_id, *index, compilation_ctx)? =>
        {
            // Get child struct by (module_id, index)
            let child_struct = compilation_ctx.get_struct_by_intermediate_type(itype)?;

            if child_struct.has_key {
                // Child struct has 'key' ability: it's stored as a separate object with its own UID.
                // When deleting the parent, we only remove the reference to the child object,
                // but the child object itself remains in storage and must be deleted separately.
            } else {
                // Child struct has no 'key' ability: it's stored inline/flattened within the parent.
                // When deleting the parent, we must also delete the child's data from storage
                // since it's not a separate object and will be orphaned.
                add_delete_storage_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    slot_ptr,
                    slot_offset,
                    &child_struct,
                )?;
            }
        }
        IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
            add_delete_storage_enum_instructions(
                module,
                builder,
                compilation_ctx,
                slot_ptr,
                slot_offset,
                itype,
            )?;
        }
        IntermediateType::IVector(inner_) => {
            // Delete the vector recursively
            // This function does not modify the original elem_slot_ptr passed as argument
            // elem_slot_ptr is copied and used as the new header slot pointer
            add_delete_storage_vector_instructions(
                module,
                builder,
                compilation_ctx,
                slot_ptr,
                inner_,
            )?;
        }
        _ => {}
    }

    Ok(())
}

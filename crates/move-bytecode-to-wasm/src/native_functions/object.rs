use super::NativeFunction;
use crate::{
    CompilationContext,
    data::DATA_SLOT_DATA_PTR_OFFSET,
    get_generic_function_name,
    hostio::host_functions::{
        block_number, block_timestamp, emit_log, native_keccak256, storage_cache_bytes32,
        storage_flush_cache, storage_load_bytes32,
    },
    runtime::RuntimeFunction,
    storage::encoding::field_size,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        heap_integers::{IU128, IU256},
        structs::IStruct,
    },
    utils::keccak_string_to_memory,
    vm_handled_types::{VmHandledType, named_id::NamedId, uid::Uid},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn add_compute_named_id_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_COMPUTE_NAMED_ID, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
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

        let struct_ = compilation_ctx
            .get_struct_by_index(module_id, *index)
            .unwrap();

        // Store the keccak256 hash of the counter key into linear memory at #counter_key_ptr
        keccak_string_to_memory(&mut builder, compilation_ctx, &struct_.identifier, id_ptr);

        // Return the ID ptr
        builder.local_get(id_ptr);

        function.finish(vec![], &mut module.funcs)
    } else {
        panic!(
            r#"there was an error linking "{}" function, expected IStruct, found {itype:?}"#,
            NativeFunction::NATIVE_COMPUTE_NAMED_ID
        );
    }
}

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

    // Iterate over the fields of the struct and delete them
    for field in struct_.fields.iter() {
        let field_size = field_size(field, compilation_ctx) as i32;
        add_delete_slot_instructions(
            module,
            builder,
            compilation_ctx,
            slot_ptr,
            field,
            field_size,
            used_bytes_in_slot,
        );
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

        block.block(None, |inner_block| {
            let inner_block_id = inner_block.id();
            inner_block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                add_delete_slot_instructions(
                    module,
                    loop_,
                    compilation_ctx,
                    elem_slot_ptr,
                    inner,
                    elem_size,
                    used_bytes_in_slot,
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
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_cache);
    });
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
/// `used_bytes_in_slot` - number of bytes already used in the current slot
fn add_delete_slot_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    itype: &IntermediateType,
    size: i32,
    used_bytes_in_slot: LocalId,
) {
    let (storage_cache, _) = storage_cache_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));

    builder
        .local_get(used_bytes_in_slot)
        .i32_const(size)
        .binop(BinaryOp::I32Add)
        .i32_const(32)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            // If used_bytes_in_slot + elem_size > 32, wipe the slot and advance the elem_slot_ptr
            |then| {
                // Wipe the slot
                then.local_get(slot_ptr)
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .call(storage_cache);

                // Calculate next slot
                then.local_get(slot_ptr)
                    .call(next_slot_fn)
                    .local_set(slot_ptr);

                // Set used_bytes_in_slot to elem_size
                then.i32_const(size).local_set(used_bytes_in_slot);
            },
            // If used_bytes_in_slot + elem_size <= 32, increment used_bytes_in_slot by elem_size
            |else_| {
                // Increment used_bytes_in_slot by elem_size
                else_
                    .local_get(used_bytes_in_slot)
                    .i32_const(size)
                    .binop(BinaryOp::I32Add)
                    .local_set(used_bytes_in_slot);
            },
        );

    match itype {
        IntermediateType::IStruct {
            module_id, index, ..
        } if !Uid::is_vm_type(module_id, *index, compilation_ctx) => {
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
                    builder,
                    compilation_ctx,
                    slot_ptr,
                    child_struct,
                    used_bytes_in_slot,
                );
            }
        }
        IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            ..
        } => {
            if !NamedId::is_vm_type(module_id, *index, compilation_ctx) {
                let child_struct = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();
                let child_struct = child_struct.instantiate(types);

                let has_key = false;
                if has_key {
                    // TODO: Implement this
                } else {
                    // If the struct does not have key, then we can delete it directly
                    add_delete_storage_struct_instructions(
                        module,
                        builder,
                        compilation_ctx,
                        slot_ptr,
                        &child_struct,
                        used_bytes_in_slot,
                    );
                }
            }
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
            );
        }
        _ => {}
    }
}

#[cfg(debug_assertions)]
pub fn add_get_last_memory_position_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    if let Some(function) = module
        .funcs
        .by_name(NativeFunction::NATIVE_GET_LAST_MEMORY_POSITION)
    {
        return function;
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let mut builder = function
        .name(NativeFunction::NATIVE_GET_LAST_MEMORY_POSITION.to_owned())
        .func_body();

    // Call allocator with size 0 to get the current memory position
    builder.i32_const(0).call(compilation_ctx.allocator);

    function.finish(vec![], &mut module.funcs)
}

/// Computes a keccak256 hash from:
/// - parent address (32 bytes)
/// - key (variable size)
/// - Key type name
///
/// Arguments
/// * `parent_address` - i32 pointer to the parent address in memory
/// * `key_ptr` - i32 pointer to the key in memory
///
/// Returns
/// * i32 pointer to the resulting hash in memory
pub fn add_hash_type_and_key_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_HASH_TYPE_AND_KEY, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let (native_keccak, _) = native_keccak256(module);

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function.name(name).func_body();

    // Arguments
    let parent_address = module.locals.add(ValType::I32);
    let key_ptr = module.locals.add(ValType::I32);

    // Locals
    let data_start = module.locals.add(ValType::I32);
    let result_ptr = module.locals.add(ValType::I32);

    // Fist we allocate space for the address
    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_set(data_start);

    builder
        .local_get(data_start)
        .local_get(parent_address)
        .i32_const(IAddress::HEAP_SIZE)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Copy the data after the parent addresss
    copy_data_to_memory(&mut builder, compilation_ctx, module, itype, key_ptr);

    let type_name = itype.get_name(compilation_ctx);

    for chunk in type_name.as_bytes() {
        builder.i32_const(1).call(compilation_ctx.allocator);

        builder.i32_const(*chunk as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
    }

    builder.local_get(data_start);

    // Call allocator to get the end of the data to Hash and substract the start to get the length
    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_get(data_start)
        .binop(BinaryOp::I32Sub);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(result_ptr);

    builder.call(native_keccak).local_get(result_ptr);

    function.finish(vec![parent_address, key_ptr], &mut module.funcs)
}

fn copy_data_to_memory(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    itype: &IntermediateType,
    data: LocalId,
) {
    // Copy the data after the parent addresss
    match itype {
        IntermediateType::IAddress => {
            builder
                .i32_const(IAddress::HEAP_SIZE)
                .call(compilation_ctx.allocator);

            builder
                .local_get(data)
                .i32_const(IAddress::HEAP_SIZE)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        // 4 bytes numbers should be in the stack
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32 => {
            builder
                .i32_const(itype.stack_data_size() as i32)
                .call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU64 => {
            builder
                .i32_const(itype.stack_data_size() as i32)
                .call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU128 => {
            builder
                .i32_const(IU128::HEAP_SIZE)
                .call(compilation_ctx.allocator);

            builder
                .local_get(data)
                .i32_const(IU128::HEAP_SIZE)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        IntermediateType::IU256 => {
            builder
                .i32_const(IU256::HEAP_SIZE)
                .call(compilation_ctx.allocator);

            builder
                .local_get(data)
                .i32_const(IU256::HEAP_SIZE)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        IntermediateType::IStruct {
            module_id, index, ..
        } => {
            let struct_ = compilation_ctx
                .get_struct_by_index(module_id, *index)
                .unwrap();

            let field_data = module.locals.add(ValType::I32);

            for (index, field) in struct_.fields.iter().enumerate() {
                builder
                    .local_get(data)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: index as u32 * 4,
                        },
                    )
                    .local_set(field_data);

                copy_data_to_memory(builder, compilation_ctx, module, field, field_data);
            }
        }
        IntermediateType::IVector(inner) => {
            let len = module.locals.add(ValType::I32);
            let i = module.locals.add(ValType::I32);
            builder
                .local_tee(data)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(len);

            let (field_data, load_kind, element_multiplier) = if **inner == IntermediateType::IU64 {
                (
                    module.locals.add(ValType::I64),
                    LoadKind::I64 { atomic: false },
                    8,
                )
            } else {
                (
                    module.locals.add(ValType::I32),
                    LoadKind::I32 { atomic: false },
                    4,
                )
            };

            builder.i32_const(1).local_set(i);
            builder.skip_vec_header(data).local_set(data);

            builder.block(None, |block| {
                let block_id = block.id();
                block.loop_(None, |loop_| {
                    let loop_id = loop_.id();

                    // Load the element pointer from the vector data
                    loop_
                        .local_get(data)
                        .i32_const(element_multiplier)
                        .local_get(i)
                        .binop(BinaryOp::I32Mul)
                        .binop(BinaryOp::I32Add)
                        .load(
                            compilation_ctx.memory_id,
                            load_kind,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(field_data);

                    copy_data_to_memory(loop_, compilation_ctx, module, inner, field_data);

                    // If we reach the last element, we exit
                    loop_
                        .local_get(i)
                        .local_get(len)
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
        }

        _ => {
            panic!(
                r#"there was an error linking "{}" function, unsupported key type {itype:?}"#,
                NativeFunction::NATIVE_HASH_TYPE_AND_KEY
            );
        }
    }
}

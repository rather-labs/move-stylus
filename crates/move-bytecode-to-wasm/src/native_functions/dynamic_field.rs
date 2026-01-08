use super::{NativeFunction, error::NativeFunctionError};
use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{SF_MODULE_NAME_DYNAMIC_FIELD, STYLUS_FRAMEWORK_ADDRESS},
    },
    data::{
        DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET,
        DATA_STORAGE_OBJECT_OWNER_OFFSET, RuntimeErrorData,
    },
    hostio::host_functions::{native_keccak256, storage_load_bytes32},
    runtime::RuntimeFunction,
    translation::intermediate_types::error::IntermediateTypeError,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        heap_integers::{IU128, IU256},
    },
    wasm_builder_extensions::WasmBuilderExtension,
};

use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

/// Adds a dynamic field for a given parent and child ID
///
/// # Arguments
/// * `parent_address` - i32 pointer to the parent object's address in memory
/// * `child_ptr` - i32 pointer to the child object's data in memory
pub fn add_child_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_ADD_CHILD_OBJECT,
        compilation_ctx,
        &[itype],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx))?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx))?;
    let save_struct_into_storage_fn = RuntimeFunction::EncodeAndSaveInStorage.get_generic(
        module,
        compilation_ctx,
        Some(runtime_error_data),
        &[itype],
    )?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);

    let mut builder = function.name(name).func_body();

    // Arguments
    let parent_address = module.locals.add(ValType::I32);
    let child_ptr = module.locals.add(ValType::I32);

    let slot_ptr = module.locals.add(ValType::I32);

    // Calculate the destiny slot
    builder
        .local_get(parent_address)
        .local_get(child_ptr)
        .call(get_id_bytes_ptr_fn)
        .call(write_object_slot_fn);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(slot_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Save the field into storage
    builder
        .local_get(child_ptr)
        .local_get(slot_ptr)
        .call(save_struct_into_storage_fn);

    Ok(function.finish(vec![parent_address, child_ptr], &mut module.funcs))
}

// TODO: Check if object exists
// TODO: Check object type
/// Borrows a dynamic field's value for a given parent and child ID
///
/// NOTE: This function is used for both `borrow_child_object` and `borrow_child_object_mut` since
/// the underlying implementation is the same. The mutability is handled at a higher level in the
/// using the type system and does not affect the WebAssembly code generation.
///
/// # Arguments
/// * `parent_uid` - i32 pointer to the parent object's UID in memory
/// * `child_id` - i32 pointer to the child ID in memory
///
/// Returns
/// * i32 pointer to a reference to the borrowed object's data in memory
pub fn add_borrow_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_BORROW_CHILD_OBJECT,
        compilation_ctx,
        &[itype],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx))?;
    let read_and_decode_from_storage_fn = RuntimeFunction::ReadAndDecodeFromStorage.get_generic(
        module,
        compilation_ctx,
        Some(runtime_error_data),
        &[itype],
    )?;

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function.name(name).func_body();

    // Arguments
    let parent_uid = module.locals.add(ValType::I32);
    let child_id = module.locals.add(ValType::I32);

    // Calculate the destiny slot
    builder
        .local_get(parent_uid)
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
        .local_tee(parent_uid)
        .local_get(child_id)
        .call(write_object_slot_fn);

    // Write the owner
    builder
        .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
        .local_get(parent_uid)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    let result_struct = module.locals.add(ValType::I32);

    let slot_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(slot_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Read from storage
    builder
        .local_get(slot_ptr)
        .local_get(child_id)
        .call(read_and_decode_from_storage_fn)
        .local_set(result_struct);

    let result = module.locals.add(ValType::I32);
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(result)
        .local_get(result_struct)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    builder.local_get(result);

    Ok(function.finish(vec![parent_uid, child_id], &mut module.funcs))
}

pub fn add_remove_child_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_REMOVE_CHILD_OBJECT,
        compilation_ctx,
        &[&itype.clone()],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    let native_borrow_child_fn = NativeFunction::get_generic(
        NativeFunction::NATIVE_BORROW_CHILD_OBJECT,
        module,
        compilation_ctx,
        Some(runtime_error_data),
        &ModuleId::new(STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_DYNAMIC_FIELD),
        &[itype.clone()],
    )?;

    // Arguments
    let parent_uid = module.locals.add(ValType::I32);
    let child_id = module.locals.add(ValType::I32);

    // Borrow the field
    builder
        .local_get(parent_uid)
        .local_get(child_id)
        .call(native_borrow_child_fn);

    // Dereference it
    builder.load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    Ok(function.finish(vec![parent_uid, child_id], &mut module.funcs))
}

/// Checks if a child object exists for a given parent and child ID
///
/// # Arguments
/// * `parent_uid` - i32 pointer to the parent object's UID in memory
/// * `child_id` - i32 pointer to the child ID in memory
///
/// Returns
/// * i32 - 1 if the child object exists, 0 otherwise
pub fn add_has_child_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_HAS_CHILD_OBJECT,
            module_id,
        ))
        .func_body();

    let (storage_load, _) = storage_load_bytes32(module);
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx))?;
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx))?;

    // Arguments
    let parent_uid = module.locals.add(ValType::I32);
    let child_id = module.locals.add(ValType::I32);

    // Calculate the destiny slot
    builder
        .local_get(parent_uid)
        .local_get(child_id)
        .call(write_object_slot_fn);

    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    builder
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(32)
        .call(is_zero_fn)
        .negate();

    Ok(function.finish(vec![parent_uid, child_id], &mut module.funcs))
}

/// Computes a keccak256 hash from:
/// * parent address (32 bytes)
/// * key (variable size)
/// * Key type name
///
/// # Arguments
/// * `parent_address` - i32 pointer to the parent address in memory
/// * `key` - i32 pointer to the key in memory
///
/// # Returns
/// * i32 pointer to the resulting hash in memory
pub fn add_hash_type_and_key_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_HASH_TYPE_AND_KEY,
        compilation_ctx,
        &[itype],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let (native_keccak, _) = native_keccak256(module);

    // Arguments
    let parent_address = module.locals.add(ValType::I32);
    let valtype = ValType::try_from(itype)?;
    let key = module.locals.add(valtype);

    let mut function =
        FunctionBuilder::new(&mut module.types, &[ValType::I32, valtype], &[ValType::I32]);

    let mut builder = function.name(name).func_body();

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
    copy_data_to_memory(&mut builder, compilation_ctx, module, itype, key)?;

    let type_name = itype.get_name(compilation_ctx)?;

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

    Ok(function.finish(vec![parent_address, key], &mut module.funcs))
}

fn copy_data_to_memory(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    itype: &IntermediateType,
    data: LocalId,
) -> Result<(), NativeFunctionError> {
    let load_value_to_stack = |field: &IntermediateType, builder: &mut InstrSeqBuilder<'_>| {
        let load_kind = field.load_kind()?;
        builder.load(
            compilation_ctx.memory_id,
            load_kind,
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        Ok::<(), IntermediateTypeError>(())
    };

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
        | IntermediateType::IU32
        | IntermediateType::IU64 => {
            builder
                .i32_const(itype.wasm_memory_data_size()?)
                .call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                itype.store_kind()?,
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
        }
        | IntermediateType::IGenericStructInstance {
            module_id, index, ..
        } => {
            let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

            let struct_ = match itype {
                IntermediateType::IGenericStructInstance { types, .. } => {
                    &struct_.instantiate(types)
                }
                _ => struct_,
            };

            let field_data_32 = module.locals.add(ValType::I32);
            let field_data_64 = module.locals.add(ValType::I64);

            for (index, field) in struct_.fields.iter().enumerate() {
                let field_data = if field == &IntermediateType::IU64 {
                    field_data_64
                } else {
                    field_data_32
                };

                builder.local_get(data).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: index as u32 * 4,
                    },
                );

                if field.is_stack_type()? {
                    load_value_to_stack(field, builder)?;
                }

                builder.local_set(field_data);

                copy_data_to_memory(builder, compilation_ctx, module, field, field_data)?;
            }
        }
        IntermediateType::IVector(inner) => {
            let len = module.locals.add(ValType::I32);
            let i = module.locals.add(ValType::I32);
            builder
                .local_get(data)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(len);

            let load_kind = inner.load_kind()?;
            let field_data = module.locals.add(ValType::try_from(&**inner)?);
            let element_multiplier = inner.wasm_memory_data_size()?;

            builder.i32_const(0).local_set(i);
            builder.skip_vec_header(data).local_set(data);

            let mut inner_result = Ok(());
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

                    inner_result =
                        copy_data_to_memory(loop_, compilation_ctx, module, inner, field_data);

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

            inner_result?;
        }

        _ => return Err(NativeFunctionError::DynamicFieldWrongKeyType(itype.clone())),
    }

    Ok(())
}

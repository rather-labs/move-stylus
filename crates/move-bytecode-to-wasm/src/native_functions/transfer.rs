use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, UnaryOp},
};

use crate::{
    CompilationContext,
    data::{
        DATA_FROZEN_OBJECTS_KEY_OFFSET, DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
        DATA_SHARED_OBJECTS_KEY_OFFSET,
    },
    get_generic_function_name,
    runtime::RuntimeFunction,
    translation::intermediate_types::IntermediateType,
    vm_handled_types::{VmHandledType, named_id::NamedId, uid::Uid},
};

use super::NativeFunction;

/// Adds the instructions to transfer an object to a recipient.
pub fn add_transfer_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_TRANSFER_OBJECT, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    // Runtime functions
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx));
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let storage_save_fn =
        RuntimeFunction::EncodeAndSaveInStorage.get_generic(module, compilation_ctx, &[itype]);
    let delete_object_fn =
        RuntimeFunction::DeleteFromStorage.get_generic(module, compilation_ctx, &[itype]);

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let struct_ptr = module.locals.add(ValType::I32);
    let recipient_ptr = module.locals.add(ValType::I32);

    // Locals
    let owner_ptr = module.locals.add(ValType::I32);
    let id_bytes_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    builder.block(None, |block| {
        let block_id = block.id();

        // Get the owner key, which is stored in the 32 bytes prefixing the struct, which can either be:
        // - An actual account address
        // - The shared objects internal key (0x1)
        // - The frozen objects internal key (0x2)
        block
            .local_get(struct_ptr)
            .i32_const(32)
            .binop(BinaryOp::I32Sub)
            .local_tee(owner_ptr);

        // Check that the object is not shared.
        block
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        // Check that the object is not frozen.
        block
            .local_get(owner_ptr)
            .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        // If the object is neither shared nor frozen, jump to the end of the block.
        block
            .binop(BinaryOp::I32Add)
            .unop(UnaryOp::I32Eqz)
            .br_if(block_id);

        block.unreachable();
    });

    // Delete the object from the owner mapping on the storage
    builder.block(None, |block| {
        let block_id = block.id();

        // Check if the owner is zero (means there's no owner, so we don't need to delete anything)
        block.local_get(owner_ptr).i32_const(32).call(is_zero_fn);

        block.br_if(block_id);

        block.local_get(struct_ptr).call(delete_object_fn);
    });

    // If the struct has wrapped objects, those objects need to be deleted from the current owner's mapping too,
    // as they are being transfered to the recipient's mapping
    add_delete_wrapped_objects_instructions(
        module,
        &mut builder,
        compilation_ctx,
        struct_ptr,
        itype,
    );

    // Update the object ownership in memory to the recipient's address
    builder
        .local_get(owner_ptr)
        .local_get(recipient_ptr)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Get the pointer to the 32 bytes holding the data of the id
    builder
        .local_get(struct_ptr)
        .call(get_id_bytes_ptr_fn)
        .local_set(id_bytes_ptr);

    // Calculate the slot number corresponding to the (recipient, struct_id) tuple
    builder
        .local_get(recipient_ptr)
        .local_get(id_bytes_ptr)
        .call(write_object_slot_fn);

    // Allocate 32 bytes for the slot pointer and copy the DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET to it
    // This is needed because DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET might be overwritten later on
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(slot_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Store the struct in the slot associated with the new owner's mapping
    builder
        .local_get(struct_ptr)
        .local_get(slot_ptr)
        .call(storage_save_fn);

    function.finish(vec![struct_ptr, recipient_ptr], &mut module.funcs)
}

/// Adds the instructions to share an object.
pub fn add_share_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_SHARE_OBJECT, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    // Runtime functions
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let storage_save_fn =
        RuntimeFunction::EncodeAndSaveInStorage.get_generic(module, compilation_ctx, &[itype]);
    let delete_object_fn =
        RuntimeFunction::DeleteFromStorage.get_generic(module, compilation_ctx, &[itype]);
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx));

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Locals
    let owner_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    builder.block(None, |block| {
        let block_id = block.id();

        block
            .local_get(struct_ptr)
            .i32_const(32)
            .binop(BinaryOp::I32Sub)
            .local_set(owner_ptr);

        // If the object is already shared, skip to the end of the block since no action is needed.
        block
            .local_get(owner_ptr)
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn)
            .br_if(block_id);

        // Emit an unreachable if the object is frozen, as it cannot be shared.
        block
            .local_get(owner_ptr)
            .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        block.if_else(
            None,
            |then| {
                // Object cannot be frozen
                then.unreachable();
            },
            |else_| {
                // Delete the object from owner mapping on the storage
                else_.block(None, |block| {
                    let block_id = block.id();

                    // Check if the owner is zero (means there's no owner, so we don't need to delete anything)
                    block.local_get(owner_ptr).i32_const(32).call(is_zero_fn);

                    block.br_if(block_id);

                    block.local_get(struct_ptr).call(delete_object_fn);
                });

                // If the struct has wrapped objects, those objects need to be deleted from the current owner's mapping too,
                // as they are being transfered to the shared mapping
                add_delete_wrapped_objects_instructions(
                    module,
                    else_,
                    compilation_ctx,
                    struct_ptr,
                    itype,
                );

                // Update the object ownership in memory to the shared objects key
                else_
                    .local_get(owner_ptr)
                    .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Calculate the slot number in the shared objects mapping
                else_
                    .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
                    .local_get(struct_ptr)
                    .call(get_id_bytes_ptr_fn)
                    .call(write_object_slot_fn);

                // Allocate 32 bytes for the slot pointer and copy the DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET to it
                // This is needed because DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET might be overwritten later on
                else_
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(slot_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Save the struct in the shared objects mapping
                else_
                    .local_get(struct_ptr)
                    .local_get(slot_ptr)
                    .call(storage_save_fn);
            },
        );
    });

    function.finish(vec![struct_ptr], &mut module.funcs)
}

/// Adds the instructions to freeze an object.
pub fn add_freeze_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_FREEZE_OBJECT, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    // Runtime functions
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let storage_save_fn =
        RuntimeFunction::EncodeAndSaveInStorage.get_generic(module, compilation_ctx, &[itype]);
    let delete_object_fn =
        RuntimeFunction::DeleteFromStorage.get_generic(module, compilation_ctx, &[itype]);
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx));

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Locals
    let owner_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    builder.block(None, |block| {
        let block_id = block.id();
        // Get the owner key, which is stored in the 32 bytes prefixing the struct, which can either be:
        // - An actual account address
        // - The shared objects internal key (0x1)
        // - The frozen objects internal key (0x2)
        block
            .local_get(struct_ptr)
            .i32_const(32)
            .binop(BinaryOp::I32Sub)
            .local_set(owner_ptr);

        // Check that the object is not shared. If so, emit an unreacheable.
        // We dont need to check if the owner is the tx sender because this is implicitly done when unpacking the struct.
        // If the object is already frozen, we skip the rest of the function. Its a no-op.

        // Verify if the object is frozen; if true, skip to the block's end since no action is needed.
        block
            .local_get(owner_ptr)
            .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        block.br_if(block_id);

        // Check if the object is shared. If so, emit an unreachable as it cannot be frozen.
        block
            .local_get(owner_ptr)
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        block.if_else(
            None,
            |then| {
                // Shared objects cannot be frozen
                then.unreachable();
            },
            |else_| {
                // Delete the object from the owner mapping on the storage
                else_.block(None, |block| {
                    let block_id = block.id();

                    // Check if the owner is zero (means there's no owner, so we don't need to delete anything)
                    block.local_get(owner_ptr).i32_const(32).call(is_zero_fn);

                    block.br_if(block_id);

                    block.local_get(struct_ptr).call(delete_object_fn);
                });

                // If the struct has wrapped objects, those objects need to be deleted from the current owner's mapping too,
                // as they are being transfered to the frozen mapping
                add_delete_wrapped_objects_instructions(
                    module,
                    else_,
                    compilation_ctx,
                    struct_ptr,
                    itype,
                );

                // Update the object ownership in memory to the frozen objects key
                else_
                    .local_get(owner_ptr)
                    .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Calculate the struct slot in the frozen objects mapping
                else_
                    .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                    .local_get(struct_ptr)
                    .call(get_id_bytes_ptr_fn)
                    .call(write_object_slot_fn);

                // Allocate 32 bytes for the slot pointer and copy the DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET to it
                else_
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(slot_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .i32_const(32)
                    .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                // Save the struct into the frozen objects mapping
                else_
                    .local_get(struct_ptr)
                    .local_get(slot_ptr)
                    .call(storage_save_fn);
            },
        );
    });

    function.finish(vec![struct_ptr], &mut module.funcs)
}

/// Helper function to delete wrapped objects from storage.
/// Useful when transfering an object with wrapped objects.
fn add_delete_wrapped_objects_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    struct_ptr: LocalId,
    itype: &IntermediateType,
) {
    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let mut offset: i32 = 0;
    // Iterate over the fields of the struct
    // If the field is a struct with key ability, delete it from storage
    for field in struct_.fields.iter() {
        match field {
            IntermediateType::IStruct {
                module_id, index, ..
            }
            | IntermediateType::IGenericStructInstance {
                module_id, index, ..
            } if !Uid::is_vm_type(module_id, *index, compilation_ctx)
                && !NamedId::is_vm_type(module_id, *index, compilation_ctx) =>
            {
                let child_struct = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .expect("struct not found");

                // If it's a generic instance, instantiate; otherwise use as-is
                let child_struct =
                    if let IntermediateType::IGenericStructInstance { types, .. } = field {
                        child_struct.instantiate(types)
                    } else {
                        child_struct.clone()
                    };

                // If the child struct has 'key' ability, we need to delete it from storage
                if child_struct.has_key {
                    let child_struct_ptr = module.locals.add(ValType::I32);
                    // Get the pointer to the child struct
                    builder
                        .local_get(struct_ptr)
                        .i32_const(offset)
                        .binop(BinaryOp::I32Add)
                        // Load the intermediate pointer to the child struct
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(child_struct_ptr);

                    // Get the delete function for the child struct
                    let delete_wrapped_object_fn = RuntimeFunction::DeleteFromStorage.get_generic(
                        module,
                        compilation_ctx,
                        &[field],
                    );

                    builder
                        .local_get(child_struct_ptr)
                        .call(delete_wrapped_object_fn);
                }
            }
            _ => {}
        }
        offset += 4;
    }
}

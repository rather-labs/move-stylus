use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, UnaryOp},
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

    builder.block(None, |block| {
        let block_id = block.id();

        // Check if the owner is zero (means there's no owner, so we don't need to delete anything)
        block.local_get(owner_ptr).i32_const(32).call(is_zero_fn);

        block.br_if(block_id);

        block.local_get(struct_ptr).call(delete_object_fn);
    });

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

    // Store the struct in the slot associated with the new owner's mapping
    builder
        .local_get(struct_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
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

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Locals
    let owner_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);

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
                else_.local_get(struct_ptr).call(delete_object_fn);

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

                // Save the struct in the shared objects mapping
                else_
                    .local_get(struct_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
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

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Locals
    let owner_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);

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
                else_.local_get(struct_ptr).call(delete_object_fn);

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

                // Save the struct into the frozen objects mapping
                else_
                    .local_get(struct_ptr)
                    .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                    .call(storage_save_fn);
            },
        );
    });

    function.finish(vec![struct_ptr], &mut module.funcs)
}

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    data::{
        DATA_FROZEN_OBJECTS_KEY_OFFSET, DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
        DATA_SHARED_OBJECTS_KEY_OFFSET,
    },
    native_functions::{object::add_delete_object_fn, storage::add_storage_save_fn},
    runtime::RuntimeFunction,
    translation::intermediate_types::structs::IStruct,
};

use super::NativeFunction;

// TODO: add_share_object_fn should be moved here

pub fn add_transfer_object_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let name = format!("{}_{hash}", NativeFunction::NATIVE_TRANSFER_OBJECT);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    // This calculates the slot number of a given (outer_key, struct_id) tupple in the objects mapping
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
    let storage_save_fn = add_storage_save_fn(hash.clone(), module, compilation_ctx, struct_);
    let add_delete_object_fn = add_delete_object_fn(hash.clone(), module, compilation_ctx, struct_);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let owner_ptr = module.locals.add(ValType::I32);
    let recipient_ptr = module.locals.add(ValType::I32);
    let struct_id_ptr = module.locals.add(ValType::I32);

    // Get the owner key, which is stored in the 32 bytes prefixing the struct, which can either be:
    // - An actual account address
    // - The shared objects internal key (0x1)
    // - The frozen objects internal key (0x2)
    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Sub)
        .local_set(owner_ptr);

    // Here we should check that the object is not frozen or shared. If it is, we throw an unreacheable.
    // We dont need to check if the owner is the tx sender because this is implicitly done when unpacking the struct.
    builder.local_get(owner_ptr);
    builder.i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET);
    builder.i32_const(32); // How many bytes to compare
    builder.call(equality_fn);

    builder.local_get(owner_ptr);
    builder.i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET);
    builder.i32_const(32);
    builder.call(equality_fn);

    builder.binop(BinaryOp::I32Or); // If the object is frozen or shared, the result will be 1 and we emit an unreacheable
    builder.if_else(
        None,
        |then| {
            then.unreachable();
        },
        |else_| {
            // Delete the object from the owner mapping on the storage
            else_.local_get(struct_ptr).call(add_delete_object_fn);

            // The first field of any struct with the key ability is its id.
            // We load the struct_ptr, so now struct_id_ptr holds a pointer to the id.
            else_
                .local_get(struct_ptr)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(struct_id_ptr);

            // Calculate the slot number corresponding to the (recipient, struct_id) tupple
            // Slot number will be written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET
            else_
                .local_get(recipient_ptr)
                .local_get(struct_id_ptr)
                .call(write_object_slot_fn);

            else_
                .local_get(struct_ptr)
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .call(storage_save_fn);
        },
    );

    function.finish(vec![struct_ptr, recipient_ptr], &mut module.funcs)
}

pub fn add_freeze_object_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let name = format!("{}_{hash}", NativeFunction::NATIVE_FREEZE_OBJECT);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    // This calculates the slot number of a given (outer_key, struct_id) tupple in the objects mapping
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));
    let storage_save_fn = add_storage_save_fn(hash.clone(), module, compilation_ctx, struct_);
    let add_delete_object_fn = add_delete_object_fn(hash.clone(), module, compilation_ctx, struct_);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let owner_ptr = module.locals.add(ValType::I32);
    let struct_id_ptr = module.locals.add(ValType::I32);

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

        // Here we should check that the object is not shared. If so, we emit an unreacheable.
        // We dont need to check if the owner is the tx sender because this is implicitly done when unpacking the struct.
        // If the object is already frozen, we skip the rest of the function. Its a no-op.

        // Check if the object is frozen
        block
            .local_get(owner_ptr)
            .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        block.br_if(block_id); // If the object is frozen, jump to the end of the block.

        // Check if the object is shared
        block
            .local_get(owner_ptr)
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .call(equality_fn);

        block.if_else(
            None,
            |then| {
                // Object cannot be shared
                then.unreachable();
            },
            |else_| {
                // Delete the object from the owner mapping on the storage
                else_.local_get(struct_ptr).call(add_delete_object_fn);

                // Get struct id
                else_
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(struct_id_ptr);

                // Calculate the struct slot in the frozen objects mapping
                else_
                    .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                    .local_get(struct_id_ptr)
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

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SHARED_OBJECTS_KEY_OFFSET},
    runtime::RuntimeFunction,
    storage,
    translation::intermediate_types::structs::IStruct,
};

use super::NativeFunction;

pub fn add_storage_save_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let name = format!("{}_{hash}", NativeFunction::NATIVE_STORAGE_SAVE);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    storage::encoding::add_encode_and_save_into_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        struct_ptr,
        slot_ptr,
        struct_,
    );

    function.finish(vec![struct_ptr, slot_ptr], &mut module.funcs)
}

pub fn add_share_object_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let name = format!("{}_{hash}", NativeFunction::NATIVE_STORAGE_SHARE_OBJECT);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let tmp = module.locals.add(ValType::I32);

    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));

    // Shared object key (owner ptr)
    builder.i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET);

    // The first field is its id, so we follow the pointer of the
    // first field
    builder
        .local_get(struct_ptr)
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
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(tmp);

    // Slot number is in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET
    builder.call(write_object_slot_fn);

    // Call storage save for the struct
    let storage_save_fn = add_storage_save_fn(hash, module, compilation_ctx, struct_);

    builder
        .local_get(struct_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(storage_save_fn);

    function.finish(vec![struct_ptr], &mut module.funcs)
}

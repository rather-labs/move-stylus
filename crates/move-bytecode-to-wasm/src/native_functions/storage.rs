use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
};

use crate::{
    CompilationContext,
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SHARED_OBJECTS_KEY_OFFSET},
    hostio::host_functions::emit_log,
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

    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let storage_save_fn = add_storage_save_fn(hash, module, compilation_ctx, struct_);
    let (emit_log_fn, _) = emit_log(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);

    // Shared object key (owner ptr)
    builder.i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET);

    // Obtain the object's id bytes pointer
    builder.local_get(struct_ptr).call(get_id_bytes_ptr_fn);

    // Slot number is in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET
    builder.call(write_object_slot_fn);

    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_fn);

    // Call storage save for the struct
    builder
        .local_get(struct_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(storage_save_fn);

    function.finish(vec![struct_ptr], &mut module.funcs)
}

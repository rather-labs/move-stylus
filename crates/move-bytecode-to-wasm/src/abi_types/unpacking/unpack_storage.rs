use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{
    CompilationContext,
    data::DATA_OBJECTS_SLOT_OFFSET,
    hostio::host_functions::tx_origin,
    runtime::RuntimeFunction,
    translation::intermediate_types::{address::IAddress, structs::IStruct},
};

/// Generates a function that looks for an specific struct in storage given its id
pub fn unpack_from_storage(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
    name: String,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let uid_ptr = module.locals.add(ValType::I32);
    let origin_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);

    // This would be the owner
    let (tx_origin, _) = tx_origin(module);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(origin_ptr)
        .call(tx_origin);

    // Calculate the slot address
    let derive_slot_fn = RuntimeFunction::DeriveMappingSlot.get(module, Some(compilation_ctx));

    // Save space to for the derive slot ptr
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(slot_ptr);

    // Derive the slot for the first mapping
    builder
        .i32_const(DATA_OBJECTS_SLOT_OFFSET)
        .local_get(origin_ptr)
        .local_get(slot_ptr)
        .call(derive_slot_fn);

    // Derive slot for ther second mapping
    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .local_get(slot_ptr)
        .call(derive_slot_fn);

    builder.local_get(struct_ptr);

    function.finish(vec![uid_ptr], &mut module.funcs)
}

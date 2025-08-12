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
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );
    let mut builder = function.name(name).func_body();

    let owner_ptr = module.locals.add(ValType::I32);
    let uid_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);

    // Calculate the slot address
    let derive_slot_fn = RuntimeFunction::DeriveMappingSlot.get(module, Some(compilation_ctx));

    // Derive the slot for the first mapping
    builder
        .i32_const(DATA_OBJECTS_SLOT_OFFSET)
        .local_get(owner_ptr)
        .local_get(slot_ptr)
        .call(derive_slot_fn);

    // Derive slot for ther second mapping
    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .local_get(slot_ptr)
        .call(derive_slot_fn);

    function.finish(vec![owner_ptr, uid_ptr, slot_ptr], &mut module.funcs)
}

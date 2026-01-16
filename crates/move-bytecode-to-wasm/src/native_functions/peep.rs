use super::NativeFunction;
use crate::{
    CompilationContext, IntermediateType, Module, ModuleId, data::DATA_SLOT_DATA_PTR_OFFSET,
    hostio::host_functions::storage_load_bytes32, native_functions::error::NativeFunctionError,
    runtime::RuntimeFunction, wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

/// This function allows to peek into the storage of another address.
///
// # WASM Function Aguments:
// * `owner_address_ptr` - pointer to the owner address
// * `uid_ptr` - pointer to the object id
//
// # WASM Function Returns:
// * reference to the object in memory
pub fn add_peep_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_PEEP,
        compilation_ctx,
        &[itype],
        module_id,
    )?;

    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let (storage_load, _) = storage_load_bytes32(module);
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx))?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx))?;

    let owner_address_ptr = module.locals.add(ValType::I32);
    let uid_ptr = module.locals.add(ValType::I32);

    let slot_ptr = module.locals.add(ValType::I32);

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Load the UID ptr from the reference
    builder
        .local_get(uid_ptr)
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
        .local_set(uid_ptr);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(slot_ptr);

    // Search for the object in the owner's address mapping
    builder.block(None, |block| {
        let exit_block = block.id();

        block
            .local_get(owner_address_ptr)
            .local_get(uid_ptr)
            .local_get(slot_ptr)
            .call(write_object_slot_fn);

        // Load data from slot
        block
            .local_get(slot_ptr)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);

        // Check if it is empty (all zeroes)
        block
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(32)
            .call(is_zero_fn)
            .negate()
            .br_if(exit_block);

        // If we get here means the object was not found
        block.unreachable();
    });

    // Decode the storage object into the internal representation

    let read_and_decode_from_storage_fn =
        RuntimeFunction::ReadAndDecodeFromStorage.get_generic(module, compilation_ctx, &[itype])?;

    // Allocate memory for the reference to the object
    let ref_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(ref_ptr);

    // Decode the object from the storage encoding into the internal representation
    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .local_get(owner_address_ptr)
        .call(read_and_decode_from_storage_fn);

    // Store the ptr to the decoded object into the reference
    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    builder.local_get(ref_ptr);

    Ok(function.finish(vec![owner_address_ptr, uid_ptr], &mut module.funcs))
}

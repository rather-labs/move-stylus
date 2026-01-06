use super::NativeFunction;
use crate::{
    CompilationContext, IntermediateType, Module, ModuleId,
    data::{
        DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_SLOT_DATA_PTR_OFFSET,
        DATA_STORAGE_OBJECT_OWNER_OFFSET,
    },
    hostio::host_functions::storage_load_bytes32,
    native_functions::error::NativeFunctionError,
    runtime::RuntimeFunction,
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{FunctionBuilder, FunctionId, ValType};

// This function allows to peek into the storage of another address.
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

    let owner_address = module.locals.add(ValType::I32);
    let uid_ptr = module.locals.add(ValType::I32);

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Search for the object in the owner's address mapping
    // If not found, emit an unreachable
    builder.block(None, |block| {
        let exit_block = block.id();

        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .local_get(uid_ptr)
            .call(write_object_slot_fn);

        // Load data from slot
        block
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
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

    Ok(function.finish(vec![owner_address, uid_ptr], &mut module.funcs))
}

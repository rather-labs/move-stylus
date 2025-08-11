use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{CompilationContext, data::DATA_U256_ONE_OFFSET};

use super::RuntimeFunction;

pub fn storage_next_slot_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::StorageNextSlot.name().to_owned())
        .func_body();

    let slot_ptr = module.locals.add(ValType::I32);

    let swap_256_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));
    let add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));

    // BE to LE ptr so we can make the addition
    builder
        .local_get(slot_ptr)
        .local_get(slot_ptr)
        .call(swap_256_fn);

    // Add one to slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_U256_ONE_OFFSET)
        .local_get(slot_ptr)
        .i32_const(32)
        .call(add_u256_fn);

    // LE to BE ptr so we can use the storage function
    builder
        .local_get(slot_ptr)
        .local_get(slot_ptr)
        .call(swap_256_fn);

    function.finish(vec![slot_ptr], &mut module.funcs)
}

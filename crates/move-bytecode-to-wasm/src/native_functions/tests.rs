//! This module hold functions used only in tests and debug builds.
#![cfg(debug_assertions)]

use super::{NativeFunction, error::NativeFunctionError};
use crate::{
    CompilationContext, compilation_context::ModuleId, runtime::RuntimeFunction,
    translation::intermediate_types::IntermediateType,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

pub fn add_get_last_memory_position_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    if let Some(function) = module
        .funcs
        .by_name(NativeFunction::NATIVE_GET_LAST_MEMORY_POSITION)
    {
        return function;
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

    let mut builder = function
        .name(NativeFunction::NATIVE_GET_LAST_MEMORY_POSITION.to_owned())
        .func_body();

    // Call allocator with size 0 to get the current memory position
    builder.i32_const(0).call(compilation_ctx.allocator);

    function.finish(vec![], &mut module.funcs)
}

pub fn add_read_slot_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::READ_SLOT,
        compilation_ctx,
        &[itype],
        module_id,
    )?;

    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    let slot_ptr = module.locals.add(ValType::I32);
    let uid_ptr = module.locals.add(ValType::I32);

    let read_and_decode_fn =
        RuntimeFunction::ReadAndDecodeFromStorage.get_generic(module, compilation_ctx, &[itype])?;

    // The owner pointer does not matter in this functions, since it is testing
    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .i32_const(0)
        .call(read_and_decode_fn);

    Ok(function.finish(vec![slot_ptr, uid_ptr], &mut module.funcs))
}

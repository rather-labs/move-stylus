//! This module hold functions used only in tests and debug builds.
#![cfg(debug_assertions)]

use super::NativeFunction;
use crate::CompilationContext;
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

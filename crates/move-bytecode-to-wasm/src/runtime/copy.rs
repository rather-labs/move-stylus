use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use super::RuntimeFunction;
use crate::{CompilationContext, translation::intermediate_types::heap_integers::IU128};

pub fn copy_heap_int_function<const SIZE: i32>(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::CopyU128.name().to_owned())
        .func_body();

    let src_ptr = module.locals.add(ValType::I32);
    let dst_ptr = module.locals.add(ValType::I32);

    builder
        .i32_const(SIZE)
        .call(compilation_ctx.allocator)
        .local_tee(dst_ptr);

    builder.local_get(src_ptr);

    builder.i32_const(SIZE);

    builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    builder.local_get(dst_ptr);

    function.finish(vec![src_ptr], &mut module.funcs)
}

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use super::RuntimeFunction;
use crate::CompilationContext;

pub fn copy_u128_function(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::CopyU128.name().to_owned())
        .func_body();

    let src_ptr = module.locals.add(ValType::I32);

    builder.i32_const(16);
    builder.call(compilation_ctx.allocator);
    let dst_ptr = module.locals.add(ValType::I32);
    builder.local_set(dst_ptr);

    for i in 0..2 {
        builder.local_get(dst_ptr).local_get(src_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: i * 8,
            },
        );
    }
    for i in 0..2 {
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 8 - i * 8,
            },
        );
    }
    builder.local_get(dst_ptr);
    function.finish(vec![src_ptr], &mut module.funcs)
}

pub fn copy_u256_function(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::CopyU256.name().to_owned())
        .func_body();

    let src_ptr = module.locals.add(ValType::I32);

    builder.i32_const(32);
    builder.call(compilation_ctx.allocator);
    let dst_ptr = module.locals.add(ValType::I32);
    builder.local_set(dst_ptr);

    for i in 0..4 {
        builder.local_get(dst_ptr).local_get(src_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: i * 8,
            },
        );
    }
    for i in 0..4 {
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 24 - i * 8,
            },
        );
    }
    builder.local_get(dst_ptr);
    function.finish(vec![src_ptr], &mut module.funcs)
}

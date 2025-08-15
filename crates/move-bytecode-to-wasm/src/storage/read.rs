use std::hash::{DefaultHasher, Hash, Hasher};

use super::{encoding::add_read_and_decode_storage_struct_instructions, get_struct};
use crate::hostio::host_functions::emit_log;
use crate::{
    CompilationContext, storage::READ_STRUCT_FROM_STORAGE_FN_NAME,
    translation::intermediate_types::IntermediateType,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

/// Generates a function that reads an specific struct from the storage.
pub fn add_read_struct_from_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let mut hasher = DefaultHasher::new();
    itype.hash(&mut hasher);
    let name = format!("{READ_STRUCT_FROM_STORAGE_FN_NAME}_{:x}", hasher.finish());
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let struct_ = get_struct!(itype, compilation_ctx);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let slot_ptr = module.locals.add(ValType::I32);

    let struct_ptr = add_read_and_decode_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        slot_ptr,
        struct_,
    );

    let (emit_log_fn, _) = emit_log(module);
    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_fn);

    builder
        .local_get(struct_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_fn);

    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Sub)
        .i32_const(32)
        .i32_const(0)
        .call(emit_log_fn);

    builder.local_get(struct_ptr);

    function.finish(vec![slot_ptr], &mut module.funcs)
}

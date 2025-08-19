use std::hash::{DefaultHasher, Hash, Hasher};

use super::encoding::add_read_and_decode_storage_struct_instructions;
use crate::{
    CompilationContext, storage::READ_STRUCT_FROM_STORAGE_FN_NAME,
    translation::intermediate_types::IntermediateType,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

// TODO: move to runtime
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

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let slot_ptr = module.locals.add(ValType::I32);

    let struct_ptr = add_read_and_decode_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        slot_ptr,
        &struct_,
    );

    builder.local_get(struct_ptr);

    function.finish(vec![slot_ptr], &mut module.funcs)
}

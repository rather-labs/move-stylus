use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    CompilationContext, storage::SAVE_STRUCT_INTO_STORAGE_FN_NAME,
    translation::intermediate_types::IntermediateType,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::encoding::add_encode_and_save_into_storage_struct_instructions;

/// Generates a function that reads an specific struct from the storage.
pub fn add_save_struct_into_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let mut hasher = DefaultHasher::new();
    itype.hash(&mut hasher);
    let name = format!("{SAVE_STRUCT_INTO_STORAGE_FN_NAME}_{:x}", hasher.finish());
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    add_encode_and_save_into_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        struct_ptr,
        slot_ptr,
        &struct_,
    );

    function.finish(vec![struct_ptr, slot_ptr], &mut module.funcs)
}

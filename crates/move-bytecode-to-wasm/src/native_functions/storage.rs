use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{CompilationContext, storage, translation::intermediate_types::structs::IStruct};

use super::NativeFunction;

pub fn add_storage_save_fn(
    hash: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let name = format!("{}_{hash}", NativeFunction::NATIVE_STORAGE_SAVE);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    storage::encoding::add_encode_and_save_into_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        struct_ptr,
        slot_ptr,
        struct_,
        0,
    );

    function.finish(vec![struct_ptr, slot_ptr], &mut module.funcs)
}

use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{CompilationContext, storage, translation::intermediate_types::structs::IStruct};

use super::NativeFunction;

pub fn add_storage_save_fn(
    name: String,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);

    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    let mut builder = function.name(name).func_body();

    storage::encode::store(
        module,
        &mut builder,
        compilation_ctx,
        struct_ptr,
        slot_ptr,
        struct_,
    );

    function.finish(vec![struct_ptr, slot_ptr], &mut module.funcs)
}

use walrus::{FunctionBuilder, Module, ValType};

use crate::{
    CompilationContext, storage::encode, translation::intermediate_types::structs::IStruct,
};

use super::NativeFunction;

pub fn add_storage_save_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

    let struct_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::NATIVE_STORAGE_SAVE.to_owned())
        .func_body();

    encode::store(module, &mut builder, compilation_ctx, struct_ptr, struct_);

    function.finish(vec![struct_ptr], &mut module.funcs)
}

use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    native_functions::error::NativeFunctionError,
    vm_handled_types::{VmHandledType, tx_context::TxContext},
};

use super::NativeFunction;

/// Adds unit tests poison function
pub fn add_poison_fn(module: &mut Module, module_id: &ModuleId) -> FunctionId {
    let name = NativeFunction::get_function_name(NativeFunction::NATIVE_POISON, module_id);

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[], &[]);
    let mut builder = function.name(name).func_body();

    builder.unreachable();

    function.finish(vec![], &mut module.funcs)
}

pub fn add_new_tx_context_fn(
    module: &mut Module,
    module_id: &ModuleId,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_function_name(NativeFunction::NATIVE_NEW_TX_CONTEXT, module_id);

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    TxContext::inject(&mut builder, module, compilation_ctx)?;

    Ok(function.finish(vec![], &mut module.funcs))
}

pub fn add_drop_storage_object_fn(module: &mut Module, module_id: &ModuleId) -> FunctionId {
    let name =
        NativeFunction::get_function_name(NativeFunction::NATIVE_DROP_STORAGE_OBJECT, module_id);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    function.name(name).func_body();

    function.finish(vec![], &mut module.funcs)
}

use walrus::{FunctionBuilder, FunctionId, Module};

use crate::compilation_context::ModuleId;

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

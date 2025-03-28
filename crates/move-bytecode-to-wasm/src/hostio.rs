use walrus::{FunctionBuilder, FunctionId, Module, ModuleConfig, ValType};

mod host_functions;

/// Create a new module with stylus mandatory `pay_for_memory_grow` function and `memory` exports
pub fn new_module_with_host() -> Module {
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);

    let memory_id = module.memories.add_local(false, false, 1, None, None);
    module.exports.add("memory", memory_id);

    host_functions::add_pay_for_memory_grow(&mut module);

    module
}

/// Add an entrypoint to the module as required by Stylus
/// TODO: This should route to the actual functions
pub fn add_entrypoint(module: &mut Module, func: FunctionId, func_name: &str) {
    let mut entrypoint = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    entrypoint.func_body().call(func).unreachable();
    let entrypoint = entrypoint.finish(vec![], &mut module.funcs);
    module.exports.add("user_entrypoint", entrypoint);
}

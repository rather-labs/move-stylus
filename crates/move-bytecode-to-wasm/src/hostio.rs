use walrus::{FunctionBuilder, FunctionId, Module, ModuleConfig, ValType};

/// Create a new module with stylus mandatory host functions and memory
pub fn new_module_with_host() -> Module {
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);
    let memory_id = module.memories.add_local(false, false, 1, None, None);
    module.exports.add("memory", memory_id);

    let pay_for_memory_grow_type = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "pay_for_memory_grow", pay_for_memory_grow_type);

    module
}

/// Add an entrypoint to the module as required by Stylus
/// TODO: This should route to the actual functions
pub fn add_entrypoint(module: &mut Module, func: FunctionId) {
    let mut entrypoint = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    entrypoint.func_body().call(func);
    let entrypoint = entrypoint.finish(vec![], &mut module.funcs);
    module.exports.add("user_entrypoint", entrypoint);
}

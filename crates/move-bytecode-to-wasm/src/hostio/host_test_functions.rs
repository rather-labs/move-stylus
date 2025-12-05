use walrus::{FunctionId, ImportId, Module, ValType};

pub fn set_sender_address(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_sender_address", &[ValType::I32], &[])
}

pub fn set_signer_address(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_signer_address", &[ValType::I32], &[])
}

#[inline]
fn get_or_insert_import(
    module: &mut walrus::Module,
    name: &str,
    params: &[walrus::ValType],
    results: &[walrus::ValType],
) -> (walrus::FunctionId, walrus::ImportId) {
    if let Ok(function_id) = module.imports.get_func("vm_test_hooks", name) {
        for import in module.imports.iter() {
            if let walrus::ImportKind::Function(func_id) = import.kind {
                if func_id == function_id {
                    return (function_id, import.id());
                }
            }
        }
    }

    let ty = module.types.add(params, results);
    module.add_import_func("vm_test_hooks", name, ty)
}

use walrus::{FunctionId, ImportId, Module, ValType};

pub const TEST_HOST_MODULE_NAME: &str = "vm_test_hooks";

pub fn set_sender_address(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_sender_address", &[ValType::I32], &[])
}

pub fn set_signer_address(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_signer_address", &[ValType::I32], &[])
}

pub fn set_block_basefee(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_block_basefee", &[ValType::I32], &[])
}

pub fn set_gas_price(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_gas_price", &[ValType::I32], &[])
}

pub fn set_block_number(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_block_number", &[ValType::I64], &[])
}

pub fn set_gas_limit(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_gas_limit", &[ValType::I64], &[])
}

pub fn set_block_timestamp(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_block_timestamp", &[ValType::I64], &[])
}

pub fn set_chain_id(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "set_chain_id", &[ValType::I64], &[])
}

#[inline]
fn get_or_insert_import(
    module: &mut walrus::Module,
    name: &str,
    params: &[walrus::ValType],
    results: &[walrus::ValType],
) -> (walrus::FunctionId, walrus::ImportId) {
    if let Ok(function_id) = module.imports.get_func(TEST_HOST_MODULE_NAME, name) {
        for import in module.imports.iter() {
            if let walrus::ImportKind::Function(func_id) = import.kind {
                if func_id == function_id {
                    return (function_id, import.id());
                }
            }
        }
    }

    let ty = module.types.add(params, results);
    module.add_import_func(TEST_HOST_MODULE_NAME, name, ty)
}

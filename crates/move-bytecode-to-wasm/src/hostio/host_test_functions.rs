use crate::utils::get_or_insert_import;
use walrus::{FunctionId, ImportId, Module, ValType};

pub const TEST_HOST_MODULE_NAME: &str = "vm_test_hooks";

pub fn set_sender_address(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_sender_address",
        &[ValType::I32],
        &[],
    )
}

pub fn set_signer_address(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_signer_address",
        &[ValType::I32],
        &[],
    )
}

pub fn set_block_basefee(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_block_basefee",
        &[ValType::I32],
        &[],
    )
}

pub fn set_gas_price(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_gas_price",
        &[ValType::I32],
        &[],
    )
}

pub fn set_block_number(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_block_number",
        &[ValType::I64],
        &[],
    )
}

pub fn set_gas_limit(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_gas_limit",
        &[ValType::I64],
        &[],
    )
}

pub fn set_block_timestamp(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_block_timestamp",
        &[ValType::I64],
        &[],
    )
}

pub fn set_chain_id(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        TEST_HOST_MODULE_NAME,
        "set_chain_id",
        &[ValType::I64],
        &[],
    )
}

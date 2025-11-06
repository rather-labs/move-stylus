use move_bytecode_to_wasm::compilation_context::ModuleId;

pub const STYLUS_FRAMEWORK_ADDRESS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
];

pub const STANDARD_LIB_ADDRESS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

// Module names for stylus framework
pub const SF_MODULE_NAME_TX_CONTEXT: &str = "tx_context";
pub const SF_MODULE_NAME_DYNAMIC_FIELD: &str = "dynamic_field";
pub const SF_MODULE_NAME_CONTRACT_CALLS: &str = "contract_calls";
pub const SF_MODULE_NAME_DYNAMIC_FIELD_NAMED_ID: &str = "dynamic_field_named_id";
pub const SF_MODULE_NAME_OBJECT: &str = "object";
pub const SF_MODULE_NAME_TRANSFER: &str = "transfer";
pub const SF_MODULE_NAME_EVENT: &str = "event";
pub const SF_MODULE_NAME_TYPES: &str = "types";
pub const SF_MODULE_NAME_TABLE: &str = "table";

// Module names for standard lib
pub const STDLIB_MODULE_NAME_ASCII: &str = "ascii";
pub const STDLIB_MODULE_NAME_STRING: &str = "string";

/// This function checks if the type is hidden from signature.
pub fn is_hidden_in_signature(identifier: &str, module_id: Option<&ModuleId>) -> bool {
    match (identifier, module_id) {
        ("NamedId", Some(module_id)) => {
            module_id.module_name == SF_MODULE_NAME_OBJECT
                && module_id.address.as_slice() == STYLUS_FRAMEWORK_ADDRESS
        }
        ("TxContext", Some(module_id)) => {
            module_id.module_name == SF_MODULE_NAME_TX_CONTEXT
                && module_id.address.as_slice() == STYLUS_FRAMEWORK_ADDRESS
        }
        ("signer", None) => true,
        _ => false,
    }
}

/// This function checks if the type is hidden from signature.
pub fn convert_type<'a>(identifier: &'a str, module_id: Option<&ModuleId>) -> &'a str {
    match (identifier, module_id) {
        ("UID", Some(module_id))
            if module_id.module_name == SF_MODULE_NAME_OBJECT
                && module_id.address.as_slice() == STYLUS_FRAMEWORK_ADDRESS =>
        {
            "bytes32"
        }
        _ => identifier,
    }
}

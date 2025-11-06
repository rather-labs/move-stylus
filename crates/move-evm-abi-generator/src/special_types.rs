use std::collections::HashMap;

use move_bytecode_to_wasm::compilation_context::{
    ModuleData, ModuleId, module_data::struct_data::IntermediateType,
};

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
pub fn convert_type<'a>(
    identifier: &'a str,
    intermediate_type: &IntermediateType,
    modules_data: &HashMap<ModuleId, ModuleData>,
) -> &'a str {
    match (identifier, intermediate_type) {
        ("UID", IntermediateType::IStruct { module_id, .. })
            if module_id.module_name == SF_MODULE_NAME_OBJECT
                && module_id.address.as_slice() == STYLUS_FRAMEWORK_ADDRESS =>
        {
            "bytes32"
        }
        (
            _,
            IntermediateType::IStruct {
                module_id, index, ..
            }
            | IntermediateType::IGenericStructInstance {
                module_id, index, ..
            },
        ) => {
            if let Some(module_data) = modules_data.get(module_id) {
                let struct_ = module_data.structs.get_by_index(*index).unwrap();
                if struct_.has_key {
                    "bytes32"
                } else {
                    identifier
                }
            } else {
                panic!("module {module_id} not found in module data")
            }
        }
        _ => identifier,
    }
}

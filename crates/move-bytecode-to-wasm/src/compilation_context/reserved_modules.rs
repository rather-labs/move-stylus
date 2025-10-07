use super::module_data::Address;

pub const STYLUS_FRAMEWORK_ADDRESS: Address = Address::from_bytes([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
]);

pub const STYLUS_FRAMEWORK_NAME: &str = "stylus";

// Module names
pub const SF_MODULE_NAME_TX_CONTEXT: &str = "tx_context";
pub const SF_MODULE_NAME_DYNAMIC_FIELD: &str = "dynamic_field";
pub const SF_MODULE_NAME_OBJECT: &str = "object";
pub const SF_MODULE_NAME_TRANSFER: &str = "transfer";
pub const SF_MODULE_NAME_EVENT: &str = "event";
pub const SF_MODULE_NAME_TYPES: &str = "types";

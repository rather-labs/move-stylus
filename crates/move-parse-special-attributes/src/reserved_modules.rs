// Standard Library Address
pub const STDLIB_ADDRESS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

// Module names for standard lib
pub const STDLIB_MODULE_NAME_ASCII: &str = "ascii";
pub const STDLIB_MODULE_NAME_STRING: &str = "string";

// Stylus Framework Address
pub const SF_ADDRESS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
];

// Stylus Framework Name
pub const SF_NAME: &str = "StylusFramework";

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
pub const SF_MODULE_NAME_ERROR: &str = "error";
pub const SF_MODULE_NAME_FALLBACK: &str = "fallback";

/// All reserved struct names in the Stylus Framework
pub const SF_RESERVED_STRUCTS: &[&str] = &[
    "TxContext",
    "Field",
    "CrossContractCall",
    "ContractCallResult",
    "ContractCallEmptyResult",
    "ID",
    "UID",
    "NamedId",
    "Table",
    "Calldata",
];

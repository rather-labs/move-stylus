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

/// Represents a Stylus Framework module with its reserved structs and events
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub name: &'static str,
    pub structs: &'static [&'static str],
    pub events: &'static [&'static str],
}

/// All Stylus Framework modules with their forbidden items
pub const STYLUS_FRAMEWORK_MODULES: &[ModuleInfo] = &[
    ModuleInfo {
        name: SF_MODULE_NAME_TX_CONTEXT,
        structs: &["TxContext"],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_DYNAMIC_FIELD,
        structs: &["Field"],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_CONTRACT_CALLS,
        structs: &[
            "CrossContractCall",
            "ContractCallResult",
            "ContractCallEmptyResult",
        ],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_DYNAMIC_FIELD_NAMED_ID,
        structs: &[],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_OBJECT,
        structs: &["ID", "UID", "NamedId"],
        events: &["NewUID"],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_TRANSFER,
        structs: &[],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_EVENT,
        structs: &[],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_TYPES,
        structs: &[],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_TABLE,
        structs: &["Table"],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_ERROR,
        structs: &[],
        events: &[],
    },
    ModuleInfo {
        name: SF_MODULE_NAME_FALLBACK,
        structs: &["Calldata"],
        events: &[],
    },
];

/// Main structure containing all Stylus Framework information
#[derive(Debug, Clone)]
pub struct StylusFrameworkPackage {
    pub name: &'static str,
    pub address: [u8; 32],
    pub modules: &'static [ModuleInfo],
}

/// Stylus Framework package instance with all modules
pub const STYLUS_FRAMEWORK: StylusFrameworkPackage = StylusFrameworkPackage {
    name: SF_NAME,
    address: SF_ADDRESS,
    modules: STYLUS_FRAMEWORK_MODULES,
};

impl StylusFrameworkPackage {
    /// Checks if a struct name is forbidden in any module
    /// Returns true if the struct exists in a different module (forbidden),
    /// false if it doesn't exist in any module or exists in the same module (allowed)
    pub fn is_reserved_struct(&self, module_name: &str, struct_name: &str) -> bool {
        for module in self.modules {
            if module.structs.contains(&struct_name) {
                // If found in a different module, it's forbidden
                return module.name != module_name;
            }
        }
        // Not found in any module, so it's not forbidden
        false
    }

    /// Checks if an event name is forbidden in any module
    pub fn is_reserved_event(&self, event_name: &str) -> Option<&'static str> {
        for module in self.modules {
            if module.events.contains(&event_name) {
                return Some(module.name);
            }
        }
        None
    }

    /// Gets module info by module name
    pub fn get_module(&self, module_name: &str) -> Option<&ModuleInfo> {
        self.modules.iter().find(|m| m.name == module_name)
    }
}

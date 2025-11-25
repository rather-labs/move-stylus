use move_bytecode_to_wasm::compilation_context::{
    ModuleId,
    reserved_modules::{
        SF_MODULE_NAME_OBJECT, STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_ASCII,
        STDLIB_MODULE_NAME_STRING, STYLUS_FRAMEWORK_ADDRESS,
    },
};

pub fn is_named_id(identifier: &str, module_id: &ModuleId) -> bool {
    "NamedId" == identifier
        && module_id.module_name == SF_MODULE_NAME_OBJECT
        && module_id.address == STYLUS_FRAMEWORK_ADDRESS
}

pub fn is_uid(identifier: &str, module_id: &ModuleId) -> bool {
    "UID" == identifier
        && module_id.module_name == SF_MODULE_NAME_OBJECT
        && module_id.address == STYLUS_FRAMEWORK_ADDRESS
}

pub fn is_string(identifier: &str, module_id: &ModuleId) -> bool {
    "String" == identifier
        && module_id.address == STANDARD_LIB_ADDRESS
        && (module_id.module_name == STDLIB_MODULE_NAME_ASCII
            || module_id.module_name == STDLIB_MODULE_NAME_STRING)
}

use move_bytecode_to_wasm::compilation_context::{
    ModuleId,
    reserved_modules::{SF_MODULE_NAME_OBJECT, STYLUS_FRAMEWORK_ADDRESS},
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

use crate::compilation_context::{ModuleId, reserved_modules::STYLUS_FRAMEWORK_ADDRESS};

pub struct Field;

impl Field {
    const BORROW_CHILD_OBJECT_MUT_IDENTIFIER: &str = "borrow_child_object_mut";
    const BORROW_MUT_IDENTIFIER: &str = "borrow_mut";

    pub const MODULE_DYNAMIC_FIELD: &str = "dynamic_field";
    pub const MODULE_DYNAMIC_FIELD_NAMED_ID: &str = "dynamic_field_named_id";

    pub fn is_borrow_child_object_mut_fn(module_id: &ModuleId, identifier: &str) -> bool {
        identifier == Self::BORROW_CHILD_OBJECT_MUT_IDENTIFIER
            && module_id.address == STYLUS_FRAMEWORK_ADDRESS
            && module_id.module_name == "dynamic_field"
    }

    pub fn is_borrow_mut_fn(module_id: &ModuleId, identifier: &str) -> bool {
        identifier == Self::BORROW_MUT_IDENTIFIER
            && module_id.address == STYLUS_FRAMEWORK_ADDRESS
            && (module_id.module_name == "dynamic_field"
                || module_id.module_name == "dynamic_field_named_id")
    }
}

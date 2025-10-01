use crate::compilation_context::{ModuleId, reserved_modules::STYLUS_FRAMEWORK_ADDRESS};

pub struct Field;

impl Field {
    const BORROW_CHILD_OBJECT_MUT_IDENTIFIER: &str = "borrow_child_object_mut";

    pub fn is_borrow_child_object_mut_fn(module_id: &ModuleId, identifier: &str) -> bool {
        identifier == Self::BORROW_CHILD_OBJECT_MUT_IDENTIFIER
            && module_id.address == STYLUS_FRAMEWORK_ADDRESS
            && module_id.module_name == "dynamic_field"
    }
}

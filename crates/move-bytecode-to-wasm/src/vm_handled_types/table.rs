use crate::compilation_context::{
    ModuleId,
    reserved_modules::{SF_MODULE_NAME_TABLE, STYLUS_FRAMEWORK_ADDRESS},
};

pub struct Table;

impl Table {
    const BORROW_MUT_IDENTIFIER: &str = "borrow_mut";

    pub fn is_borrow_mut_fn(module_id: &ModuleId, identifier: &str) -> bool {
        identifier == Self::BORROW_MUT_IDENTIFIER
            && module_id.address == STYLUS_FRAMEWORK_ADDRESS
            && module_id.module_name == SF_MODULE_NAME_TABLE
    }
}

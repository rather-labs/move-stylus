// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use crate::compilation_context::{
    ModuleId,
    reserved_modules::{
        SF_MODULE_NAME_DYNAMIC_FIELD, SF_MODULE_NAME_DYNAMIC_FIELD_NAMED_ID,
        STYLUS_FRAMEWORK_ADDRESS,
    },
};

pub struct Field;

impl Field {
    const BORROW_MUT_IDENTIFIER: &str = "borrow_mut";

    pub fn is_borrow_mut_fn(module_id: &ModuleId, identifier: &str) -> bool {
        identifier == Self::BORROW_MUT_IDENTIFIER
            && module_id.address == STYLUS_FRAMEWORK_ADDRESS
            && (module_id.module_name.as_str() == SF_MODULE_NAME_DYNAMIC_FIELD
                || module_id.module_name.as_str() == SF_MODULE_NAME_DYNAMIC_FIELD_NAMED_ID)
    }
}

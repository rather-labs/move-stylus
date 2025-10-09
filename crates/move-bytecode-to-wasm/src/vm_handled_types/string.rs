use super::VmHandledType;
use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{
            STANDARD_LIB_ADDRESS, STDLIB_MODULE_NAME_ASCII, STDLIB_MODULE_NAME_STRING,
        },
    },
};
use walrus::{InstrSeqBuilder, Module};

pub struct String_;

impl VmHandledType for String_ {
    const IDENTIFIER: &str = "String";

    fn inject(
        _block: &mut InstrSeqBuilder,
        _module: &mut Module,
        _compilation_ctx: &CompilationContext,
    ) {
        // String are not injected, they are created by the user
    }

    fn is_vm_type(module_id: &ModuleId, index: u16, compilation_ctx: &CompilationContext) -> bool {
        let identifier = &compilation_ctx
            .get_struct_by_index(module_id, index)
            .unwrap()
            .identifier;

        if identifier == Self::IDENTIFIER {
            if module_id.address != STANDARD_LIB_ADDRESS
                || (module_id.module_name != STDLIB_MODULE_NAME_ASCII
                    && module_id.module_name != STDLIB_MODULE_NAME_STRING)
            {
                panic!("invalid String found, only the one from the standard lib is valid");
            }
            return true;
        }
        false
    }
}

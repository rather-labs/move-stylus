//! This module contains all the functions retaled to transaction information.

use walrus::{FunctionId, Module, ValType};

pub fn add_native_sender_fn(module: &mut Module) -> FunctionId {
    let msg_sender = module.types.add(&[], &[ValType::I32]);
    let (function_id, _) = module.add_import_func("vm_hooks", "msg_sender", msg_sender);
    function_id
}

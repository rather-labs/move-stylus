//! This module contains all the functions retaled to transaction information.

use walrus::{FunctionId, Module, ValType};

use crate::hostio::host_functions::msg_sender;

pub fn add_native_sender_fn(module: &mut Module) -> FunctionId {
    let (function_id, _) = msg_sender(module);
    function_id
}

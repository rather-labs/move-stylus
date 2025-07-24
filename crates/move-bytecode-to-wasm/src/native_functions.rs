//! This module contains the implementation for the native functions.
//!
//! Native functions in Move are functions directly implemented inside the Move VM. To emulate that
//! mechanism, we direcly implement them in WASM and limk them into the file.
mod transaction;

use walrus::{FunctionId, Module};

use crate::CompilationContext;

pub struct NativeFunction;

impl NativeFunction {
    const NATIVE_SENDER: &str = "native_sender";

    /// Links the function into the module and returns its id. If the function is already present
    /// it just returns the id.
    ///
    /// This funciton is idempotent.
    pub fn get(name: &str, module: &mut Module, compilaton_ctx: &CompilationContext) -> FunctionId {
        if let Some(function) = module.funcs.by_name(name) {
            function
        } else {
            match name {
                Self::NATIVE_SENDER => transaction::add_native_sender_fn(module, compilaton_ctx),
                _ => panic!("native function {name} not supported yet"),
            }
        }
    }
}

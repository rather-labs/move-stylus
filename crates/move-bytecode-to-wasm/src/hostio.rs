#[cfg(test)]
pub use entrypoint_router::add_entrypoint;
pub use entrypoint_router::build_entrypoint_router;
use walrus::{Module, ModuleConfig};

use crate::memory::initialize_module_memory;

pub mod entrypoint_router;
pub mod host_functions;

/// Create a new module with stylus memory management functions and adds the `pay_for_memory_grow` function
/// as required by stylus
pub fn new_module_with_host() -> Module {
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);

    initialize_module_memory(&mut module);
    host_functions::add_pay_for_memory_grow(&mut module);

    module
}

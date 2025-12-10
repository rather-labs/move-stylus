#[cfg(test)]
pub use entrypoint_router::add_entrypoint;
pub use entrypoint_router::build_entrypoint_router;
use walrus::{FunctionId, MemoryId, Module, ModuleConfig};

use crate::{
    data::{TOTAL_RESERVED_MEMORY, setup_data_segment},
    memory::setup_module_memory,
};

pub mod entrypoint_router;
pub mod error;
pub mod host_functions;
pub mod host_test_functions;

/// Create a new module with stylus memory management functions and adds the `pay_for_memory_grow` function
/// as required by stylus
pub fn new_module_with_host() -> (Module, FunctionId, MemoryId) {
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);

    let (allocator_function_id, memory_id) =
        setup_module_memory(&mut module, Some(TOTAL_RESERVED_MEMORY));
    let (memory_grow_id, _) = host_functions::add_pay_for_memory_grow(&mut module);

    // We re-export it so the optimizer does not erases it
    module.exports.add("pfmg", memory_grow_id);

    // Fill data segment
    setup_data_segment(&mut module, memory_id);

    (module, allocator_function_id, memory_id)
}

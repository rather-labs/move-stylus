use std::path::Path;

use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};

pub fn run_tests(module_id: &ModuleId, module_data: &ModuleData, module_path: &Path) {
    println!(
        "Running tests for {module_id} ({})...\n",
        module_path.display()
    );

    for test in &module_data.special_attributes.test_functions {
        println!("Running {test}...")
    }
}

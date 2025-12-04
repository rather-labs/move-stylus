mod constants;
mod wasm_runner;

use std::path::Path;

use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use wasm_runner::RuntimeSandbox;

pub fn run_tests(
    module_id: &ModuleId,
    module_data: &ModuleData,
    module_path: &Path,
    compiled_modules_path: &Path,
) {
    println!(
        "Running tests for {module_id} ({})...\n",
        module_path.display()
    );

    let mut compiled_wasm = compiled_modules_path
        .to_path_buf()
        .join(&module_id.module_name);
    compiled_wasm.set_extension("wasm");

    for test in &module_data.special_attributes.test_functions {
        print!("Running {test}... ");
        let runtime = RuntimeSandbox::new(&compiled_wasm);
        let result = runtime.call_test_function(test);

        if result.is_ok() {
            println!("\x1B[1m\x1B[32mOK\x1B[0m");
        } else {
            println!("\x1B[1m\x1B[31mFAIL\x1B[0m");
        }
    }
}

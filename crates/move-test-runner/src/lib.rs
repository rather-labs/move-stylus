mod constants;
mod wasm_runner;

use std::path::Path;

use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use wasm_runner::RuntimeSandbox;

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";

pub fn run_tests(
    module_id: &ModuleId,
    module_data: &ModuleData,
    module_path: &Path,
    compiled_modules_path: &Path,
) {
    println!(
        "\nRunning {CYAN}{module_id}{RESET} tests ({})]\n",
        module_path.display()
    );

    let mut compiled_wasm = compiled_modules_path
        .to_path_buf()
        .join(&module_id.module_name);
    compiled_wasm.set_extension("wasm");

    let mut failures = Vec::new();
    for test in &module_data.special_attributes.test_functions {
        print!("  {module_id}::{test} ... ");
        let runtime = RuntimeSandbox::new(&compiled_wasm);
        let result = runtime.call_test_function(test);
        if result.is_ok() {
            println!("{GREEN}PASSED{RESET}");
        } else {
            println!("{RED}FAILED{RESET}");
            failures.push(test.to_owned());
        }
    }

    let total = module_data.special_attributes.test_functions.len();

    print!(
        "\nTotal Tests : {}, ",
        module_data.special_attributes.test_functions.len()
    );
    print!("{GREEN}Passed{RESET}: {}, ", total - failures.len(),);
    println!("{RED}Failed{RESET}: {}.", failures.len());

    if !failures.is_empty() {
        println!("Failed tests:");
        for failed_test in failures {
            println!("  {module_id}::{failed_test}");
        }
    }
}

mod constants;
mod wasm_runner;

use std::path::{Path, PathBuf};

use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use wasm_runner::RuntimeSandbox;

pub struct Summary {
    module: PathBuf,
    failed_tests: Vec<String>,
}

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";

pub fn run_tests(
    module_id: &ModuleId,
    module_data: &ModuleData,
    module_path: &Path,
    compiled_modules_path: &Path,
) -> Summary {
    println!(
        "\n[ RUNNING {CYAN}{module_id}{RESET} TESTS ({})]",
        module_path.display()
    );

    let mut compiled_wasm = compiled_modules_path
        .to_path_buf()
        .join(&module_id.module_name);
    compiled_wasm.set_extension("wasm");

    println!("------------------------------------------------------------");
    let mut failures = Vec::new();
    for test in &module_data.special_attributes.test_functions {
        let runtime = RuntimeSandbox::new(&compiled_wasm);
        let result = runtime.call_test_function(test);

        if result.is_ok() {
            println!("+ {module_id}::{test:50} {GREEN}PASSED{RESET}");
        } else {
            println!("- {module_id}::{test:50} {RED}FAILED{RESET}");
            failures.push(test.to_owned());
        }
    }
    println!("------------------------------------------------------------");

    let total = module_data.special_attributes.test_functions.len();
    println!("\n[ SUMMARY ]",);
    println!("------------------------------------------------------------");
    println!(
        "Total Tests : {}",
        module_data.special_attributes.test_functions.len()
    );
    println!("{GREEN}Passed{RESET}      : {}", total - failures.len(),);
    println!("{RED}Failed{RESET}      : {}", failures.len());
    println!("------------------------------------------------------------");

    Summary {
        module: module_path.to_path_buf(),
        failed_tests: failures,
    }
}

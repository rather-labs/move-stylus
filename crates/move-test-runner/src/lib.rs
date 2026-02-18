// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

pub mod constants;
pub mod wasm_runner;

use std::path::Path;

use move_bytecode_to_wasm::compilation_context::{ModuleData, ModuleId};
use move_parse_special_attributes::function_modifiers::ExpectedAbortCode;
use wasm_runner::RuntimeSandbox;

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";

/// Mask that zeroes out bits 47-32 of a u64 (the line number in clever errors).
///
/// Clever error encoding (see `ErrorBitset` in `move-command-line-common/src/error_bitset.rs`):
/// ```text
/// |<tagbit>|<reserved>|<line number>|<identifier index>|<constant index>|
///   1-bit    15-bits       16-bits        16-bits          16-bits
/// ```
/// The line number varies depending on where a macro function is expanded, so we mask it
/// out when comparing expected vs actual clever error abort codes.
const CLEVER_ERROR_COMPARISON_MASK: u64 = 0xFFFF_0000_FFFF_FFFF;

/// Compares two abort codes. First tries an exact match. If that fails and both
/// are clever errors (tag bit `0x8000` set), masks out the line number (bits 47-32)
/// and compare again.
fn abort_codes_match(expected: u64, actual: u64) -> bool {
    if expected == actual {
        return true;
    }

    if ((expected >> 48) != 0x8000) || ((actual >> 48) != 0x8000) {
        return false;
    }

    (expected & CLEVER_ERROR_COMPARISON_MASK) == (actual & CLEVER_ERROR_COMPARISON_MASK)
}

pub fn run_tests(
    module_id: &ModuleId,
    module_data: &ModuleData,
    module_path: &Path,
    compiled_modules_path: &Path,
) -> bool {
    println!(
        "\nRunning {CYAN}{module_id}{RESET} tests ({})\n",
        module_path.display()
    );

    let mut compiled_wasm = compiled_modules_path
        .to_path_buf()
        .join(module_id.module_name.as_str());
    compiled_wasm.set_extension("wasm");

    let mut failures = Vec::new();
    for test in &module_data.special_attributes.test_functions {
        print!(
            "  {module_id}::{} {}... ",
            test.name,
            if test.expect_failure {
                "[expected failure] "
            } else {
                ""
            }
        );
        let runtime = RuntimeSandbox::from_path(&compiled_wasm);

        let result = runtime.call_test_function(&test.name).unwrap();
        match (result.execution_aborted, test.expect_failure) {
            (false, true) => {
                println!("{RED}FAILED{RESET} - expected test to abort but it succeeded");
                failures.push(test.to_owned());
            }
            (false, false) => {
                println!("{GREEN}PASSED{RESET}");
            }
            (true, false) => {
                println!("{RED}FAILED{RESET}");
                failures.push(test.to_owned());
            }
            (true, true) => {
                // Execution aborted as expected — now check the abort code if specified
                match &test.expected_abort_code {
                    Some(ExpectedAbortCode::Literal(expected_code)) => {
                        if let Some(actual_code) = result.store.data().abort_code {
                            if abort_codes_match(*expected_code, actual_code) {
                                println!("{GREEN}PASSED{RESET}");
                            } else {
                                println!(
                                    "{RED}FAILED{RESET} - expected abort code {expected_code} (0x{expected_code:X}), got {actual_code} (0x{actual_code:X})"
                                );
                                failures.push(test.to_owned());
                            }
                        } else {
                            println!(
                                "{YELLOW}PASSED{RESET} (abort code could not be extracted for comparison)"
                            );
                        }
                    }
                    Some(ExpectedAbortCode::Constant(module_name, constant_name)) => {
                        // This is unreachable because we have throw an error in the compiler if any constant reference cannot be resolved.
                        println!(
                            "{RED}FAILED{RESET} (abort code {module_name}::{constant_name} could not be resolved for comparison)"
                        );
                    }
                    None => {
                        // No specific abort code expected — any abort is fine
                        println!("{GREEN}PASSED{RESET}");
                    }
                }
            }
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
        for failed_test in &failures {
            println!("  {module_id}::{}", failed_test.name);
        }
    }

    !failures.is_empty()
}

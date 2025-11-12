//! This module is in charge of processing the move-bytecode-to-wasm errors

use std::process::exit;

use move_bytecode_to_wasm::error::CompilationError;
use move_compiler::diagnostics::{Diagnostics, report_diagnostics};

const GITHUB_URL: &str = "https://github.com/rather-labs/move-stylus-poc";

pub(crate) fn print_error_diagnostic(error: CompilationError) -> ! {
    match error {
        CompilationError::ICE(iceerror) => {
            eprintln!(
                r#"
                An Internal Compiler Error (ICE) has ocurred:

                {iceerror}

                Please open an issue in our GitHub repository:
                {GITHUB_URL}
                "#
            );
            exit(1)
        }

        CompilationError::CodeError {
            mapped_files,
            errors,
        } => {
            let mut diagnostics = Diagnostics::new();
            for error in errors {
                diagnostics.add(error.into());
            }
            report_diagnostics(&mapped_files, diagnostics);
        }
    }
}

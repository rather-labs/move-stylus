// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This module is in charge of processing the move-bytecode-to-wasm errors

use std::{backtrace::BacktraceStatus, error::Error};

use move_bytecode_to_wasm::error::CompilationError;
use move_compiler::diagnostics::{Diagnostics, report_diagnostics};
use move_evm_abi_generator::error::AbiGeneratorError;

const GITHUB_URL: &str = "https://github.com/rather-labs/move-stylus";

const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");
const CLI_NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) trait PrintDiagnostic {
    fn print_error_diagnostic(self);
}

impl PrintDiagnostic for AbiGeneratorError {
    fn print_error_diagnostic(self) {
        eprintln!(
            "\x1B[1m\x1B[31mAn Internal Compiler Error (ICE) has ocurred\x1B[0m: {}\n",
            self.kind
        );

        if let Some(source) = self.kind.source() {
            eprintln!("Caused by:");

            let mut current = Some(source);
            let mut index = 1;
            while let Some(cause) = current {
                eprintln!("\t{index}. {cause}");
                current = cause.source();
                index += 1;
            }
        }

        eprintln!(
            "\n\x1B[1m\x1B[34mNOTE\x1B[0m: {CLI_NAME} {CLI_VERSION} on {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );

        eprintln!("\n\x1B[1m\x1B[34mNOTE\x1B[0m: we would appreciate a bug report: {GITHUB_URL}");
    }
}

impl PrintDiagnostic for CompilationError {
    fn print_error_diagnostic(self) {
        match &self {
            CompilationError::ICE(iceerror) => {
                eprintln!(
                    "\x1B[1m\x1B[31mAn Internal Compiler Error (ICE) has ocurred\x1B[0m: {iceerror}\n"
                );

                if let Some(err) = iceerror.source() {
                    eprintln!("Caused by:");

                    let mut current = err.source();
                    let mut index = 1;
                    while let Some(cause) = current {
                        eprintln!("\t{index}. {cause}");
                        current = cause.source();
                        index += 1;
                    }
                }

                match iceerror.backtrace.status() {
                    BacktraceStatus::Unsupported => (),
                    BacktraceStatus::Disabled => eprintln!(
                        "\n\x1B[1m\x1B[34mNOTE\x1B[0m: please enable the Rust backtrace (RUST_BACKTRACE=1) before submitting an issue."
                    ),
                    BacktraceStatus::Captured => {
                        eprintln!("\nBackcktrace:\n{}", iceerror.backtrace)
                    }
                    _ => (),
                };

                eprintln!(
                    "\n\x1B[1m\x1B[34mNOTE\x1B[0m: {CLI_NAME} {CLI_VERSION} - {} {} on {} {}",
                    iceerror.name,
                    iceerror.version,
                    std::env::consts::OS,
                    std::env::consts::ARCH
                );

                eprintln!(
                    "\n\x1B[1m\x1B[34mNOTE\x1B[0m: we would appreciate a bug report: {GITHUB_URL}"
                );
            }
            CompilationError::CodeError {
                mapped_files,
                errors,
            } => {
                let mut diagnostics = Diagnostics::new();
                for error in errors {
                    diagnostics.add(error.into());
                }
                report_diagnostics(mapped_files, diagnostics);
            }
            CompilationError::NoFilesFound => {
                eprintln!("\x1B[1m\x1B[31mError:\x1B[0m no input files found to compile.");
                eprintln!(
                    "\n\x1B[1m\x1B[34mNOTE\x1B[0m: If there are source files in the project, this is an internal error and we would appreciate a bug report: {GITHUB_URL}"
                );
            }
        }
    }
}

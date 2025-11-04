use clap::*;
use move_compiler::diagnostics::{Diagnostics, report_diagnostics};
use move_evm_abi_generator::generate_abi;
use move_package::BuildConfig;
use std::path::Path;

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "abi-generate")]
pub struct AbiGenerate;

impl AbiGenerate {
    pub fn execute(self, path: Option<&Path>, _config: BuildConfig) -> anyhow::Result<()> {
        if let Err((mapped_files, errors)) = generate_abi(path.unwrap()) {
            let mut diagnostics = Diagnostics::new();
            for error in &errors {
                diagnostics.add(error.into());
            }

            report_diagnostics(&mapped_files, diagnostics)
        }
        Ok(())
    }
}

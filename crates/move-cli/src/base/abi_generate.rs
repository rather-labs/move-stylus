use clap::*;
use move_bytecode_to_wasm::{
    error::{CompilationError, CompilationErrorKind},
    package_module_data,
};
use move_compiler::diagnostics::{Diagnostics, report_diagnostics};
use move_evm_abi_generator::generate_abi;
use move_package::BuildConfig;
use std::path::Path;

use super::reroot_path;

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "abi-generate")]
pub struct AbiGenerate;

impl AbiGenerate {
    pub fn execute(self, path: Option<&Path>, config: BuildConfig) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(path)?;

        let package = config.compile_package(&rerooted_path, &mut Vec::new())?;

        let package_modules = match package_module_data(package, None) {
            Ok(package_modules) => package_modules,
            Err(CompilationError { files, kind }) => match kind {
                CompilationErrorKind::ICE(_iceerror) => todo!(),
                CompilationErrorKind::CodeError(code_errors) => {
                    let mut diagnostics = Diagnostics::new();
                    for error in &code_errors {
                        diagnostics.add(error.into());
                    }

                    report_diagnostics(&files, diagnostics)
                }
            },
        };

        match generate_abi(&rerooted_path, &package_modules) {
            Ok(mut processed_abis) => {
                let build_directory = rerooted_path.join("build/wasm");
                // Create the build directory if it doesn't exist
                std::fs::create_dir_all(&build_directory).unwrap();

                println!("{build_directory:?}");

                for abi in &mut processed_abis {
                    // Change the extension
                    abi.file.set_extension("abi");
                    let file = abi.file.file_name().expect("file not found");
                    println!("asd {:?}", build_directory.join(&abi.file));
                    std::fs::write(build_directory.join(file), abi.content.as_bytes())?;
                }
            }
            Err((mapped_files, errors)) => {
                let mut diagnostics = Diagnostics::new();
                for error in &errors {
                    diagnostics.add(error.into());
                }

                report_diagnostics(&mapped_files, diagnostics)
            }
        }

        Ok(())
    }
}

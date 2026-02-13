// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use clap::*;
use move_bytecode_to_wasm::package_module_data;
use move_evm_abi_generator::generate_abi;
use move_package::{BuildConfig, compilation::compiled_package::CompiledUnitWithSource};
use std::{path::Path, process::exit};

use crate::error::PrintDiagnostic;

use super::reroot_path;

/// Generate the package ABI at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "export-abi")]
pub struct ExportAbi {
    /// Generate JSON format ABI files
    #[clap(long = "json", short = 'j')]
    pub json: bool,

    /// Generate human-readable ABI files (.sol)
    #[clap(long = "human-readable", short = 'r')]
    pub human_readable: bool,
}

impl ExportAbi {
    pub fn execute(
        self,
        path: Option<&Path>,
        module_name: Option<String>,
        config: BuildConfig,
        verbose: bool,
    ) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(path)?;
        let install_dir = config.install_dir.clone();

        let package = config.compile_package(&rerooted_path, &mut Vec::new())?;
        let root_compiled_units: Vec<&CompiledUnitWithSource> =
            if let Some(module_name) = module_name {
                package
                    .root_compiled_units
                    .iter()
                    .filter(move |unit| unit.unit.name.to_string() == module_name)
                    .collect()
            } else {
                package.root_compiled_units.iter().collect()
            };

        let package_modules =
            match package_module_data(&package, &root_compiled_units, verbose, false) {
                Ok(pm) => pm,
                Err(e) => {
                    e.print_error_diagnostic();
                    exit(1);
                }
            };

        // If neither flag is set, default to generating JSON
        let generate_json = self.json || !self.human_readable;
        let generate_human_readable = self.human_readable;

        match generate_abi(
            &package,
            &root_compiled_units,
            &package_modules,
            generate_json,
            generate_human_readable,
        ) {
            Ok(mut processed_abis) => {
                let build_directory = if let Some(install_dir) = install_dir {
                    install_dir.join(format!(
                        "build/{}/abi",
                        package.compiled_package_info.package_name
                    ))
                } else {
                    rerooted_path.join(format!(
                        "build/{}/abi",
                        package.compiled_package_info.package_name
                    ))
                };

                // Create the build directory if it doesn't exist
                std::fs::create_dir_all(&build_directory).unwrap();

                for abi in &mut processed_abis {
                    if generate_human_readable {
                        if let Some(content) = &abi.content_human_readable {
                            // Change the extension
                            abi.file.set_extension("sol");
                            let file = abi.file.file_name().expect("Source file name not found.");
                            std::fs::write(build_directory.join(file), content.as_bytes())?;
                        }
                    }
                    if generate_json {
                        if let Some(content) = &abi.content_json {
                            // Change the extension
                            abi.file.set_extension("json");
                            let file = abi.file.file_name().expect("Source file name not found.");
                            std::fs::write(build_directory.join(file), content.as_bytes())?;
                        }
                    }
                }
            }

            Err(abi_error) => {
                abi_error.print_error_diagnostic();
                exit(1);
            }
        }

        Ok(())
    }
}

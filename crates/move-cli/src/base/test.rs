// Copyright (c) The Move Contributors
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::error::print_error_diagnostic;

use super::{get_build_directory, reroot_path, translate_package_cli};
use clap::*;
use move_bytecode_to_wasm::package_module_data;
use move_package::{BuildConfig, compilation::compiled_package::CompiledUnitWithSource};
use move_test_runner::run_tests;
use std::{path::Path, process::exit};

/// Compiles modules and run the unit tests
#[derive(Parser)]
#[clap(name = "test")]
pub struct Test;

impl Test {
    pub fn execute(
        self,
        path: Option<&Path>,
        mut config: BuildConfig,
        verbose: bool,
    ) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(path)?;
        let install_dir = config.install_dir.clone();

        config.test_mode = true;
        config.dev_mode = true;

        let package = config.compile_package(&rerooted_path, &mut Vec::new())?;

        let build_directory = get_build_directory(&rerooted_path, &package, &install_dir);

        if let Err(compilation_error) = translate_package_cli(
            &package,
            &rerooted_path,
            install_dir,
            false,
            verbose,
            false,
            true,
        ) {
            print_error_diagnostic(*compilation_error)
        }

        let root_compiled_units: Vec<&CompiledUnitWithSource> =
            package.root_compiled_units.iter().collect();

        let package_modules = package_module_data(&package, &root_compiled_units, verbose, true)
            .map_err(print_error_diagnostic)?;

        let mut test_failed = false;
        for (path, module_id) in &package_modules.modules_paths {
            let data = package_modules.modules_data.get(module_id).unwrap();

            if !data.special_attributes.test_functions.is_empty() {
                test_failed = test_failed || run_tests(module_id, data, path, &build_directory);
            }
        }

        if test_failed {
            exit(1)
        }

        Ok(())
    }
}

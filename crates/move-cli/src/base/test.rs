// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::print_error_diagnostic;

use super::{reroot_path, translate_package_cli};
use clap::*;
use move_bytecode_to_wasm::package_module_data;
use move_package::BuildConfig;
use move_test_runner::run_tests;
use std::path::Path;

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

        let package_modules =
            package_module_data(&package, None, verbose).map_err(print_error_diagnostic)?;

        if let Err(compilation_error) =
            translate_package_cli(package, &rerooted_path, install_dir, false, verbose)
        {
            print_error_diagnostic(*compilation_error)
        }

        for (path, module_id) in &package_modules.modules_paths {
            let data = package_modules.modules_data.get(module_id).unwrap();

            if !data.special_attributes.test_functions.is_empty() {
                run_tests(module_id, data, path);
            }
        }

        Ok(())
    }
}

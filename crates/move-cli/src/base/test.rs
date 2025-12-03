// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::print_error_diagnostic;

use super::{reroot_path, translate_package_cli};
use clap::*;
use move_bytecode_to_wasm::package_module_data;
use move_package::BuildConfig;
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

        config.test_mode = true;
        config.dev_mode = true;

        let package = config.clone().cli_compile_package(
            &rerooted_path,
            &mut std::io::stdout(),
            &mut std::io::stdin().lock(),
        )?;

        let package_modules =
            package_module_data(&package, None, verbose).map_err(print_error_diagnostic)?;

        if let Err(compilation_error) =
            translate_package_cli(package, &rerooted_path, config.install_dir, false, verbose)
        {
            print_error_diagnostic(*compilation_error)
        }

        Ok(())
    }
}

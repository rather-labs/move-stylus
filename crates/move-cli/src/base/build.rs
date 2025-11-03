// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::print_error_diagnostic;

use super::reroot_path;
use clap::*;
use move_bytecode_to_wasm::translate_package_cli;
use move_package::BuildConfig;
use std::path::Path;

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "build")]
pub struct Build;

impl Build {
    pub fn execute(self, path: Option<&Path>, config: BuildConfig) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(path)?;
        if config.fetch_deps_only {
            let mut config = config;
            if config.test_mode {
                config.dev_mode = true;
            }
            config.download_deps_for_package(&rerooted_path, &mut std::io::stdout())?;
            return Ok(());
        }

        let compiled = config.clone().cli_compile_package(
            &rerooted_path,
            &mut std::io::stdout(),
            &mut std::io::stdin().lock(),
        )?;

        if let Err(compilation_error) = translate_package_cli(compiled, &rerooted_path) {
            print_error_diagnostic(compilation_error)
        }

        Ok(())
    }
}

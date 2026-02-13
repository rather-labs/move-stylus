// Copyright (c) The Move Contributors
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
// Portions of this file were modified by Rather Labs, Inc on 2025-2026.

use crate::error::PrintDiagnostic;

use super::{reroot_path, translate_package_cli};
use clap::*;
use move_package::BuildConfig;
use std::{path::Path, process::exit};

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "build")]
pub struct Build {
    /// Emits the WebAssembly Text Format along with the compiled files
    #[clap(long = "emit-wat", default_value = "false")]
    emit_wat: bool,
}

impl Build {
    pub fn execute(
        self,
        path: Option<&Path>,
        config: BuildConfig,
        verbose: bool,
    ) -> anyhow::Result<()> {
        let Build { emit_wat } = self;
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

        if let Err(compilation_error) = translate_package_cli(
            &compiled,
            &rerooted_path,
            config.install_dir,
            emit_wat,
            verbose,
            !config.dev_mode,
            false,
        ) {
            (*compilation_error).print_error_diagnostic();
            exit(1);
        }

        Ok(())
    }
}

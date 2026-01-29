// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
pub mod base;
pub(crate) mod build_config;
pub(crate) mod error;

use base::{
    activate::Activate, build::Build, deploy::Deploy, export_abi::ExportAbi, info::Info, new::New,
    test::Test,
};

#[cfg(debug_assertions)]
use base::disassemble::Disassemble;

/// Default directory where saved Move resources live
pub const DEFAULT_STORAGE_DIR: &str = "storage";

/// Default directory for build output
pub const DEFAULT_BUILD_DIR: &str = ".";

use build_config::BuildConfig;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Move {
    /// Path to a package which the command should be run with respect to.
    #[clap(long = "path", short = 'p', global = true)]
    pub package_path: Option<PathBuf>,

    /// Print additional diagnostics if available.
    #[clap(short = 'v', global = true)]
    pub verbose: bool,

    /// Package build options
    #[clap(flatten)]
    pub build_config: BuildConfig,
}

/// MoveCLI is the CLI that will be executed by the `move-cli` command
/// The `cmd` argument is added here rather than in `Move` to make it
/// easier for other crates to extend `move-cli`
#[derive(Parser)]
pub struct MoveCLI {
    #[clap(flatten)]
    pub move_args: Move,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Activate(Activate),
    ExportAbi(ExportAbi),
    Build(Build),
    #[cfg(debug_assertions)]
    Disassemble(Disassemble),
    Deploy(Deploy),
    Info(Info),
    New(New),
    Test(Test),
}

pub fn run_cli(move_args: Move, cmd: Command) -> Result<()> {
    let build_config = move_package::BuildConfig::from(move_args.build_config);

    match cmd {
        Command::ExportAbi(c) => c.execute(
            move_args.package_path.as_deref(),
            None,
            build_config,
            move_args.verbose,
        ),
        Command::Activate(c) => c.execute(),
        Command::Build(c) => c.execute(
            move_args.package_path.as_deref(),
            build_config,
            move_args.verbose,
        ),
        #[cfg(debug_assertions)]
        Command::Disassemble(c) => c.execute(
            move_args.package_path.as_deref(),
            build_config,
            move_args.verbose,
        ),
        Command::Info(c) => c.execute(move_args.package_path.as_deref(), build_config),
        Command::New(c) => c.execute_with_defaults(move_args.package_path.as_deref()),
        Command::Deploy(c) => c.execute(move_args.package_path.as_deref()),
        Command::Test(c) => c.execute(
            move_args.package_path.as_deref(),
            build_config,
            move_args.verbose,
        ),
    }
}

fn main() -> Result<()> {
    let args = MoveCLI::parse();
    run_cli(args.move_args, args.cmd)
}

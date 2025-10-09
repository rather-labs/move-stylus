use clap::*;
use move_evm_abi_generator::generate_abi;
use move_package::BuildConfig;
use std::path::Path;

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "abi-generate")]
pub struct AbiGenerate;

impl AbiGenerate {
    pub fn execute(self, path: Option<&Path>, _config: BuildConfig) -> anyhow::Result<()> {
        generate_abi(path);
        Ok(())
    }
}

use anyhow::anyhow;
use clap::Parser;
use std::process::Command;

use crate::base::cargo_stylus_installed;

/// Activates a contract
#[derive(Parser)]
#[clap(name = "activate")]
pub struct Activate {
    /// Activateed Stylus contract address to activate
    #[clap(long = "address")]
    address: String,

    /// Arbitrum RPC endpoint [default: http://localhost:8547]
    #[clap(long = "endpoint", default_value = "http://localhost:8547")]
    endpoint: String,

    /// Whether to print debug info
    #[clap(long = "verbose", default_value = "false")]
    verbose: bool,

    /// Only perform gas estimation
    #[clap(long = "estimate-gas", default_value = "false")]
    estimate_gas: bool,

    /// Optional max fee per gas in gwei units
    #[clap(long = "max-fee-per-gas-gwei", value_name = "<MAX_FEE_PER_GAS_GWEI>")]
    max_fee_per_gas_gwei: Option<String>,

    #[clap(flatten)]
    private_key: PrivateKeyArgs,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct PrivateKeyArgs {
    /// Private key as a hex string. Warning: this exposes your key to shell history
    #[clap(long = "private-key")]
    private_key: Option<String>,

    /// File path to a text file containing a hex-encoded private key
    #[clap(long = "private-key-path")]
    private_key_path: Option<String>,
}

impl Activate {
    pub fn execute(self) -> anyhow::Result<()> {
        let Self {
            address,
            endpoint,
            private_key,
            verbose,
            estimate_gas,
            max_fee_per_gas_gwei,
        } = self;

        if !cargo_stylus_installed() {
            return Err(anyhow!(
                "cargo stylus is not installed.\nPlease follow this guide to install it: https://docs.arbitrum.io/stylus/using-cli"
            ));
        }

        println!(
            "Activating contract address '{address}' to endpoint '{endpoint}' using provided private key...",
        );

        let mut command = Command::new("cargo-stylus");
        command
            .arg("--")
            .arg("activate")
            .arg("--address")
            .arg(&address)
            .arg("--endpoint")
            .arg(&endpoint);

        if verbose {
            command.arg("--verbose");
        }

        if estimate_gas {
            command.arg("--estimate-gas");
        }

        if let Some(max_fee_per_gas_gwei) = max_fee_per_gas_gwei {
            command
                .arg("--max-fee-per-gas-gwei")
                .arg(max_fee_per_gas_gwei);
        }

        match private_key {
            PrivateKeyArgs {
                private_key: Some(key),
                ..
            } => {
                command.arg("--private-key").arg(key);
            }
            PrivateKeyArgs {
                private_key_path: Some(path),
                ..
            } => {
                command.arg("--private-key-path").arg(path);
            }
            _ => {}
        }

        let result = command.output()?;
        if result.status.success() {
            println!("{}", String::from_utf8_lossy(&result.stdout));
            println!("Contract activated successfully.");
        } else {
            eprintln!(
                "Failed to activate contract. Error: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }

        Ok(())
    }
}

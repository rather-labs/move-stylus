use anyhow::anyhow;
use clap::Parser;
use std::process::Command;

/// Deploys a contract
#[derive(Parser)]
#[clap(name = "deploy")]
pub struct Deploy {
    /// Contract's name to be deployed. The .move extension is optional.
    #[clap(long = "contract-name")]
    contract_name: String,

    /// Arbitrum RPC endpoint [default: http://localhost:8547]
    #[clap(long = "endpoint", default_value = "http://localhost:8547")]
    endpoint: String,

    /// Whether to print debug info
    #[clap(long = "verbose", default_value = "false")]
    verbose: bool,

    /// Only perform gas estimation
    #[clap(long = "estimate-gas", default_value = "false")]
    estimate_gas: bool,

    /// If set, do not activate the program after deploying it
    #[clap(long = "no-activate", default_value = "false")]
    no_activate: bool,

    /// If set, do not activate the program after deploying it
    #[clap(long = "max-fee-per-gas-gwei", value_name = "<MAX_FEE_PER_GAS_GWEI>")]
    max_fee_per_gas_gwei: Option<String>,

    /// Percent to bump the estimated activation data fee by [default: 20]
    #[clap(
        long = "data-fee-bump-percent",
        value_name = "<DATA_FEE_BUMP_PERCENT>",
        default_value = "20"
    )]
    data_fee_bump_percent: String,

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

impl Deploy {
    pub fn execute(self) -> anyhow::Result<()> {
        let Self {
            contract_name,
            endpoint,
            private_key,
            verbose,
            estimate_gas,
            no_activate,
            max_fee_per_gas_gwei,
            data_fee_bump_percent,
        } = self;

        if !cargo_stylus_installed() {
            return Err(anyhow!(
                "cargo stylus is not installed.\nPlease follow this guide to install it: https://docs.arbitrum.io/stylus/using-cli"
            ));
        }

        println!(
            "Deploying contract '{contract_name}' to endpoint '{endpoint}' using provided private key...",
        );

        let mut command = Command::new("cargo-stylus");
        command
            .arg("--")
            .arg("deploy")
            .arg("--wasm-file")
            .arg(get_wasm_file_with_path(&contract_name)?)
            .arg("--endpoint")
            .arg(&endpoint)
            .arg("--data-fee-bump-percent")
            .arg(data_fee_bump_percent)
            .arg("--no-verify");

        if verbose {
            command.arg("--verbose");
        }

        if estimate_gas {
            command.arg("--estimate-gas");
        }

        if no_activate {
            command.arg("--no-activate");
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
            println!("Contract deployed successfully.");
        } else {
            eprintln!(
                "Failed to deploy contract. Error: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }

        Ok(())
    }
}

fn get_wasm_file_with_path(contract_name: &str) -> Result<String, anyhow::Error> {
    let name = if contract_name.ends_with(".move") {
        contract_name.replace(".move", ".wasm")
    } else {
        format!("{contract_name}.wasm")
    };

    let file_path = format!("./build/wasm/{name}");

    //Check if the file exists
    if !std::path::Path::new(&file_path).exists() {
        return Err(anyhow!(
            "WASM file not found at path: \"{file_path}\". Did you run \"move build\"?"
        ));
    }

    Ok(file_path)
}

fn cargo_stylus_installed() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("command -v cargo-stylus > /dev/null")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

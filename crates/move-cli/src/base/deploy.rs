use alloy::primitives::{Address, B256, U256, utils::parse_ether};
use anyhow::anyhow;
use clap::Parser;
use std::path::Path;
use std::path::PathBuf;

use crate::base::reroot_path;
use crate::common::{AuthOpts, GasFeeConfig};
use crate::deploy::STYLUS_DEPLOYER_ADDRESS;

/// Deploys a contract
#[derive(Parser, Clone, Debug)]
#[clap(name = "deploy")]
pub struct Deploy {
    /// Contract's name to be deployed. The .move extension is optional.
    #[clap(long = "contract-name")]
    pub contract_name: String,

    /// Arbitrum RPC endpoint [default: http://localhost:8547]
    #[clap(long = "endpoint", default_value = "http://localhost:8547")]
    pub endpoint: String,

    /// Whether to print debug info
    #[clap(long = "verbose", default_value = "false")]
    pub verbose: bool,

    /// Only perform gas estimation
    #[clap(long = "estimate-gas", default_value = "false")]
    pub estimate_gas: bool,

    /// If set, do not activate the program after deploying it
    #[clap(long = "no-activate", default_value = "false")]
    pub no_activate: bool,

    /// Optional max fee per gas in gwei units
    #[clap(long = "max-fee-per-gas-gwei", value_name = "<MAX_FEE_PER_GAS_GWEI>")]
    pub max_fee_per_gas_gwei: Option<String>,

    /// Percent to bump the estimated activation data fee by [default: 20]
    #[arg(long, default_value = "20")]
    pub data_fee_bump_percent: u64,

    /// The address of the deployer contract that deploys, activates, and initializes the stylus constructor.
    #[arg(long, value_name = "DEPLOYER_ADDRESS", default_value_t = STYLUS_DEPLOYER_ADDRESS)]
    pub deployer_address: Address,

    /// The salt passed to the stylus deployer.
    #[arg(long, default_value_t = B256::ZERO)]
    pub deployer_salt: B256,

    /// The constructor arguments.
    #[arg(long, num_args(0..), value_name = "ARGS", allow_hyphen_values = true)]
    pub constructor_args: Vec<String>,

    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    pub wasm_file: Option<PathBuf>,

    /// Where to deploy and activate the contract (defaults to a random address).
    #[arg(long)]
    pub contract_address: Option<Address>,

    #[clap(flatten)]
    pub auth: AuthOpts,
}

impl GasFeeConfig for Deploy {
    fn get_fee_str(&self) -> &Option<String> {
        &self.max_fee_per_gas_gwei
    }

    fn get_max_fee_per_gas_wei(&self) -> anyhow::Result<Option<u128>> {
        match self.get_fee_str() {
            Some(fee_str) => Ok(Some(crate::common::convert_gwei_to_wei(fee_str)?)),
            None => Ok(None),
        }
    }
}

impl Deploy {
    pub fn execute(mut self, path: Option<&Path>) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(path)?;
        let manifest =
            move_package::source_package::manifest_parser::parse_move_manifest_from_file(
                &rerooted_path.join("Move.toml"),
            )?;

        println!(
            "Deploying contract '{}' to endpoint '{}' using provided private key...",
            self.contract_name, self.endpoint,
        );

        if self.wasm_file.is_none() {
            self.wasm_file = Some(get_wasm_file_with_path(
                &self.contract_name,
                manifest.package.name.as_str(),
            )?);
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move {
            crate::deploy::deploy(self).await.unwrap();
        });

        Ok(())
    }
}

fn get_wasm_file_with_path(
    contract_name: &str,
    package_name: &str,
) -> Result<PathBuf, anyhow::Error> {
    let name = if contract_name.ends_with(".move") {
        contract_name.replace(".move", ".wasm")
    } else {
        format!("{contract_name}.wasm")
    };

    let file_path = format!("./build/{package_name}/wasm/{name}");

    //Check if the file exists
    if !std::path::Path::new(&file_path).exists() {
        return Err(anyhow!(
            "WASM file not found at path: \"{file_path}\". Did you run \"move build\"?"
        ));
    }

    Ok(file_path.into())
}

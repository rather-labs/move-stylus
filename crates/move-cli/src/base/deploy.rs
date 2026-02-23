use alloy::primitives::{Address, U256};
use anyhow::anyhow;
use clap::Parser;
use std::path::Path;
use std::path::PathBuf;

use crate::base::reroot_path;
use crate::deploy::{
    CheckConfig, DataFeeOpts, DeployConfig, STYLUS_DEPLOYER_ADDRESS,
};
use crate::common::{AuthOpts, CommonConfig};

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

    /// Optional max fee per gas in gwei units
    #[clap(long = "max-fee-per-gas-gwei", value_name = "<MAX_FEE_PER_GAS_GWEI>")]
    max_fee_per_gas_gwei: Option<String>,

    /// Percent to bump the estimated activation data fee by [default: 20]
    #[arg(long, default_value = "20")]
    data_fee_bump_percent: u64,

    #[clap(flatten)]
    private_key: PrivateKeyArgs,

    /// The address of the deployer contract that deploys, activates, and initializes the stylus constructor.
    #[arg(long, value_name = "DEPLOYER_ADDRESS", default_value_t = STYLUS_DEPLOYER_ADDRESS)]
    deployer_address: Address,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct PrivateKeyArgs {
    /// Private key as a hex string. Warning: this exposes your key to shell history
    #[clap(long = "private-key")]
    private_key: Option<String>,

    /// File path to a text file containing a hex-encoded private key
    #[clap(long = "private-key-path")]
    private_key_path: Option<PathBuf>,
}

fn from_deploy_args(deploy: Deploy, wasm_file: PathBuf) -> DeployConfig {
    let Deploy {
        contract_name: _,
        endpoint,
        private_key,
        verbose,
        estimate_gas,
        no_activate,
        max_fee_per_gas_gwei,
        data_fee_bump_percent,
        deployer_address,
    } = deploy;

    let PrivateKeyArgs {
        private_key,
        private_key_path,
    } = private_key;

    let auth = if private_key.is_some() {
        AuthOpts {
            private_key_path: None,
            private_key,
        }
    } else if private_key_path.is_some() {
        AuthOpts {
            private_key_path,
            private_key: None,
        }
    } else {
        panic!("Either --private-key or --private-key-path must be provided");
    };

    let check_config = CheckConfig {
        common_cfg: CommonConfig {
            endpoint,
            verbose,
            source_files_for_project_hash: vec![],
            max_fee_per_gas_gwei,
            features: None,
        },
        data_fee: DataFeeOpts {
            data_fee_bump_percent,
        },
        contract_address: None,
        wasm_file: Some(wasm_file),
    };

    DeployConfig {
        check_config,
        auth,
        estimate_gas,
        no_verify: true,
        no_activate,
        cargo_stylus_version: None,
        deployer_address,
        deployer_salt: alloy::primitives::FixedBytes::<32>::default(),
        constructor_args: vec![],
        constructor_value: U256::ZERO,
        constructor_signature: None,
    }
}

impl Deploy {
    pub fn execute(self, path: Option<&Path>) -> anyhow::Result<()> {
        let Self {
            contract_name,
            endpoint,
            ..
        } = &self;

        let rerooted_path = reroot_path(path)?;
        let manifest =
            move_package::source_package::manifest_parser::parse_move_manifest_from_file(
                &rerooted_path.join("Move.toml"),
            )?;

        println!(
            "Deploying contract '{contract_name}' to endpoint '{endpoint}' using provided private key...",
        );

        let wasm_file = get_wasm_file_with_path(contract_name, manifest.package.name.as_str())?;
        let deploy_config = from_deploy_args(self, wasm_file);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move {
            crate::deploy::deploy(deploy_config).await.unwrap();
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

use alloy::primitives::{Address, B256, U256, utils::parse_ether};
use anyhow::anyhow;
use clap::Parser;
use std::path::Path;
use std::path::PathBuf;

use crate::base::reroot_path;
use crate::common::{AuthOpts, CommonConfig};
use crate::deploy::{CheckConfig, DataFeeOpts, DeployConfig, STYLUS_DEPLOYER_ADDRESS};

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

    /// The address of the deployer contract that deploys, activates, and initializes the stylus constructor.
    #[arg(long, value_name = "DEPLOYER_ADDRESS", default_value_t = STYLUS_DEPLOYER_ADDRESS)]
    deployer_address: Address,

    /// If specified, will not run the command in a reproducible docker container. Useful for local
    /// builds, but at the risk of not having a reproducible contract for verification purposes.
    #[arg(long)]
    no_verify: bool,

    /// Cargo stylus version when deploying reproducibly to download the corresponding
    /// cargo-stylus-base Docker image. If not set, uses the default version of the local
    /// cargo stylus binary.
    #[arg(long)]
    cargo_stylus_version: Option<String>,

    /// The salt passed to the stylus deployer.
    #[arg(long, default_value_t = B256::ZERO)]
    deployer_salt: B256,

    /// The constructor arguments.
    #[arg(long, num_args(0..), value_name = "ARGS", allow_hyphen_values = true)]
    constructor_args: Vec<String>,

    /// The amount of Ether sent to the contract through the constructor.
    #[arg(long, value_parser = parse_ether, default_value = "0")]
    constructor_value: U256,

    /// The constructor signature when using the --wasm-file flag.
    #[arg(long)]
    constructor_signature: Option<String>,

    #[clap(flatten)]
    auth: AuthOpts,
}

impl Deploy {
    pub fn execute(self, path: Option<&Path>) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(path)?;
        let manifest =
            move_package::source_package::manifest_parser::parse_move_manifest_from_file(
                &rerooted_path.join("Move.toml"),
            )?;

        println!(
            "Deploying contract '{}' to endpoint '{}' using provided private key...",
            self.contract_name, self.endpoint,
        );

        let wasm_file =
            get_wasm_file_with_path(&self.contract_name, manifest.package.name.as_str())?;

        let Deploy {
            contract_name: _,
            endpoint,
            verbose,
            estimate_gas,
            no_activate,
            max_fee_per_gas_gwei,
            data_fee_bump_percent,
            deployer_address,
            no_verify,
            cargo_stylus_version,
            deployer_salt,
            constructor_args,
            constructor_value,
            constructor_signature,
            auth,
        } = self;

        let deploy_config = DeployConfig {
            check_config: CheckConfig {
                common_cfg: CommonConfig {
                    endpoint,
                    verbose,
                    max_fee_per_gas_gwei,
                },
                data_fee: DataFeeOpts {
                    data_fee_bump_percent,
                },
                contract_address: None,
                wasm_file: Some(wasm_file),
            },
            auth,
            estimate_gas,
            no_verify,
            no_activate,
            cargo_stylus_version,
            deployer_address,
            deployer_salt,
            constructor_args,
            constructor_value,
            constructor_signature,
        };

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

// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use alloy::{
    primitives::{Address, utils::format_units},
    providers::{Provider, ProviderBuilder},
    sol,
};
use clap::Parser;
use eyre::Result;
use std::path::PathBuf;

use crate::{common::{AuthOpts, CommonConfig}, constants::ARB_WASM_ADDRESS, deploy::{
     DataFeeOpts, check::check_activate,
    greyln, util::color::DebugColor,
}};

sol! {
    #[sol(rpc)]
    interface ArbWasm {
        function activateProgram(address program)
            external
            payable
            returns (uint16 version, uint256 dataFee);
    }
}

/// Activates a contract
#[derive(Parser)]
#[clap(name = "activate")]
pub struct Activate {
    /// Deployed Stylus contract address to activate
    #[clap(long = "address")]
    address: Address,

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
    private_key_path: Option<PathBuf>,
}

impl Activate {
    pub fn execute(self) -> anyhow::Result<()> {
        let Self {
            address,
            endpoint,
            ..
        } = &self;

        println!(
            "Activating contract address '{address}' to endpoint '{endpoint}' using provided private key...",
        );

        let activate_config = from_deploy_args(self);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move { activate_contract(&activate_config).await.unwrap() });
        Ok(())
    }
}

pub struct ActivateConfig {
    common_cfg: CommonConfig,

    data_fee: DataFeeOpts,

    auth: AuthOpts,

    address: Address,

    estimate_gas: bool,
}

fn from_deploy_args(activate: Activate) -> ActivateConfig {
    let Activate {
        endpoint,
        private_key,
        verbose,
        estimate_gas,
        max_fee_per_gas_gwei,
        address,
    } = activate;

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

    let common_cfg = CommonConfig {
        endpoint,
        verbose,
        source_files_for_project_hash: vec![],
        max_fee_per_gas_gwei,
        features: None,
    };

    ActivateConfig {
        common_cfg,
        data_fee: DataFeeOpts {
            data_fee_bump_percent: 20,
        },
        auth,
        address,
        estimate_gas,
    }
}

/// Activates an already deployed Stylus contract by address.
pub async fn activate_contract(cfg: &ActivateConfig) -> Result<()> {
    let provider = ProviderBuilder::new()
        .connect(&cfg.common_cfg.endpoint)
        .await?;
    let chain_id = provider.get_chain_id().await?;
    let wallet = cfg.auth.alloy_wallet(chain_id)?;
    let from_address = wallet.default_signer().address();
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&cfg.common_cfg.endpoint)
        .await?;

    let code = provider.get_code_at(cfg.address).await?;
    let data_fee = check_activate(code, cfg.address, &cfg.data_fee, &provider).await?;

    let arbwasm = ArbWasm::new(ARB_WASM_ADDRESS, &provider);
    let activate_call = arbwasm
        .activateProgram(cfg.address)
        .from(from_address)
        .value(data_fee);

    if cfg.estimate_gas {
        let gas = activate_call.estimate_gas().await?;
        let gas_price = provider.get_gas_price().await?;
        greyln!("estimates");
        greyln!("activation tx gas: {}", gas.debug_lavender());
        greyln!(
            "gas price: {} gwei",
            format_units(gas_price, "gwei")?.debug_lavender()
        );
        let total_cost = gas_price.checked_mul(gas.into()).unwrap_or_default();
        let eth_estimate = format_units(total_cost, "ether")?;
        greyln!(
            "activation tx total cost: {} ETH",
            eth_estimate.debug_lavender()
        );
    }
    let tx = activate_call.send().await?;
    let receipt = tx.get_receipt().await?;
    greyln!(
        "successfully activated contract 0x{} with tx {}",
        hex::encode(cfg.address),
        hex::encode(receipt.transaction_hash).debug_lavender()
    );
    Ok(())
}

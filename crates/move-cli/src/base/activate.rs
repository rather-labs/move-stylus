// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use alloy::{
    primitives::{Address, utils::format_units},
    providers::{Provider, ProviderBuilder},
    sol,
};
use anyhow::Result;
use clap::Parser;

use crate::{
    common::AuthOpts,
    constants::ARB_WASM_ADDRESS,
    deploy::{check::check_activate, greyln, util::color::DebugColor},
};

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

    /// Percent to bump the estimated activation data fee by [default: 20]
    #[clap(long = "data-fee-bump-percent", default_value = "20")]
    data_fee_bump_percent: u64,

    #[clap(flatten)]
    auth: AuthOpts,
}

impl Activate {
    pub fn execute(self) -> anyhow::Result<()> {
        let Self {
            address, endpoint, ..
        } = &self;

        println!(
            "Activating contract address '{address}' to endpoint '{endpoint}' using provided private key...",
        );

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move { activate_contract(&self).await.unwrap() });
        Ok(())
    }
}

/// Activates an already deployed Stylus contract by address.
pub async fn activate_contract(cfg: &Activate) -> Result<()> {
    let provider = ProviderBuilder::new().connect(&cfg.endpoint).await?;
    let chain_id = provider.get_chain_id().await?;
    let wallet = cfg.auth.alloy_wallet(chain_id)?;
    let from_address = wallet.default_signer().address();
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&cfg.endpoint)
        .await?;

    let code = provider.get_code_at(cfg.address).await?;
    let data_fee = check_activate(code, cfg.address, cfg.data_fee_bump_percent, &provider).await?;

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
